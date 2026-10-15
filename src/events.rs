//! The event loop: draw, wait for a key or a tick, refresh the Session list.

use crate::app::App;
use crate::attach::attach_command;
use crate::parse;
use crate::terminal::{self, Tui};
use crate::tmux::Tmux;
use crate::ui;
use ratatui::crossterm::event::{self, Event};
use std::io;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How often the Session list is re-read when no key is pressed.
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub fn run(terminal: &mut Tui, tmux: &Tmux, mut app: App) -> io::Result<()> {
    app.shell = std::env::var("SHELL")
        .ok()
        .and_then(|s| parse::shell_basename(&s));
    app.set_result(tmux.list_sessions());
    while !app.should_quit {
        app.now = unix_now();
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        if event::poll(REFRESH_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
                run_pending(tmux, &mut app);
                if let Some(id) = app.attach_request.take() {
                    attach(terminal, tmux, &mut app, &id)?;
                }
            }
        } else {
            app.set_result(tmux.list_sessions());
        }
    }
    Ok(())
}

/// Runs the tmux change a Popup asked for, then refreshes the list and
/// reports the outcome. Does nothing when no change is pending.
pub fn run_pending(tmux: &Tmux, app: &mut App) {
    if let Some(action) = app.pending_action.take() {
        let result = action.run(tmux);
        app.set_result(tmux.list_sessions());
        app.finish_action(result);
    }
}

/// Hands the terminal to tmux, waits for it to exit, then re-enters the TUI.
fn attach(terminal: &mut Tui, tmux: &Tmux, app: &mut App, id: &str) -> io::Result<()> {
    let (program, args) = attach_command(app.iterm2_mode, tmux.socket(), id);
    terminal::restore()?;
    let status = Command::new(program).args(args).status();
    *terminal = terminal::setup()?;
    app.message = match status {
        Ok(status) if status.success() => None,
        Ok(status) => Some(format!("tmux exited with {status}")),
        Err(error) => Some(format!("Could not run tmux: {error}")),
    };
    app.set_result(tmux.list_sessions());
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::tests::TestServer;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Keys in, screen and real tmux state out, against a throwaway server.
    struct Rig {
        server: TestServer,
        app: App,
    }

    impl Rig {
        fn new(socket: &str, sessions: &[&str]) -> Self {
            let server = TestServer::start(socket);
            for name in sessions {
                server.tmux(&["new-session", "-d", "-s", name]);
            }
            let mut app = App::default();
            app.launch_dir = "/tmp".into();
            app.set_result(server.0.list_sessions());
            Rig { server, app }
        }

        fn key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
            self.app.handle_key(KeyEvent::new(code, modifiers));
            run_pending(&self.server.0, &mut self.app);
        }

        fn press(&mut self, code: KeyCode) {
            self.key(code, KeyModifiers::NONE);
        }

        fn ctrl(&mut self, c: char) {
            self.key(KeyCode::Char(c), KeyModifiers::CONTROL);
        }

        fn type_str(&mut self, text: &str) {
            for c in text.chars() {
                self.press(KeyCode::Char(c));
            }
        }

        fn backspace(&mut self, times: usize) {
            for _ in 0..times {
                self.press(KeyCode::Backspace);
            }
        }

        fn names(&self) -> Vec<String> {
            let sessions = self.server.0.list_sessions().unwrap();
            sessions.into_iter().map(|s| s.name).collect()
        }

        fn selected(&self) -> String {
            self.app.selected_session().unwrap().name.clone()
        }

        fn screen(&mut self) -> String {
            let mut terminal = Terminal::new(TestBackend::new(140, 20)).unwrap();
            terminal.draw(|f| ui::render(f, &mut self.app)).unwrap();
            let buffer = terminal.backend().buffer();
            (0..buffer.area.height)
                .map(|y| {
                    (0..buffer.area.width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    #[test]
    fn rename_keeps_the_selection_and_never_touches_the_neighbour() {
        let mut rig = Rig::new("tmuxs-test-rename", &["cat", "category", "dog"]);
        rig.press(KeyCode::Down);
        rig.ctrl('r');
        let text = rig.screen();
        assert!(text.contains("Rename session"), "{text}");
        assert!(text.contains("New name:  category"), "{text}");
        rig.backspace(4);
        rig.type_str("gories");
        rig.press(KeyCode::Enter);
        assert_eq!(rig.names(), ["cat", "categories", "dog"]);
        assert!(rig.app.popup.is_none());
        assert_eq!(rig.selected(), "categories");
        assert!(rig.screen().contains("Renamed"));
        // F2 works too, and a rename of `cat` leaves `categories` alone.
        rig.press(KeyCode::Up);
        rig.press(KeyCode::F(2));
        rig.type_str("2");
        rig.press(KeyCode::Enter);
        assert_eq!(rig.names(), ["cat2", "categories", "dog"]);
    }

    #[test]
    fn rename_errors_stay_inside_the_popup_until_fixed() {
        let mut rig = Rig::new("tmuxs-test-rename-err", &["cat", "dog"]);
        rig.ctrl('r');
        rig.backspace(3);
        rig.press(KeyCode::Enter);
        assert!(rig.screen().contains("The name cannot be empty."));
        rig.type_str("dog");
        rig.press(KeyCode::Enter);
        let text = rig.screen();
        assert!(text.contains("duplicate session: dog"), "{text}");
        assert!(text.contains("Rename session"));
        assert_eq!(rig.names(), ["cat", "dog"]);
        rig.backspace(3);
        rig.type_str("cow");
        rig.press(KeyCode::Enter);
        assert!(rig.app.popup.is_none());
        assert_eq!(rig.names(), ["cow", "dog"]);
    }

    #[test]
    fn a_session_that_vanished_is_reported_in_the_popup_and_the_list_refreshes() {
        let mut rig = Rig::new("tmuxs-test-vanished", &["cat", "dog"]);
        rig.ctrl('r');
        rig.server.tmux(&["kill-session", "-t", "=cat"]);
        rig.press(KeyCode::Char('x'));
        rig.press(KeyCode::Enter);
        let text = rig.screen();
        assert!(text.contains("can't find session"), "{text}");
        assert!(rig.app.popup.is_some());
        assert_eq!(rig.app.sessions.len(), 1);
    }

    #[test]
    fn rename_and_kill_do_nothing_without_a_selected_session() {
        let mut rig = Rig::new("tmuxs-test-no-selection", &[]);
        rig.ctrl('r');
        rig.press(KeyCode::Delete);
        assert!(rig.app.popup.is_none());
    }

    #[test]
    fn kill_needs_y_and_spares_the_prefix_neighbour() {
        let mut rig = Rig::new("tmuxs-test-kill", &["cat", "category", "dog"]);
        rig.ctrl('x');
        let text = rig.screen();
        assert!(text.contains(r#"Kill session "cat"?"#), "{text}");
        rig.press(KeyCode::Enter);
        assert!(rig.app.popup.is_none(), "Enter is the default No");
        assert_eq!(rig.names().len(), 3);
        rig.press(KeyCode::Delete);
        rig.press(KeyCode::Char('n'));
        rig.press(KeyCode::Delete);
        rig.press(KeyCode::Esc);
        assert!(!rig.app.should_quit, "Esc closed the Popup, not the app");
        assert_eq!(rig.names().len(), 3);
        rig.press(KeyCode::Delete);
        rig.press(KeyCode::Char('y'));
        assert_eq!(rig.names(), ["category", "dog"]);
        assert_eq!(rig.selected(), "category");
        assert!(rig.screen().contains("Killed"));
    }

    #[test]
    fn kill_confirmation_warns_about_iterm2_tabs_only_in_iterm2_mode() {
        for (mode, warned) in [(true, true), (false, false)] {
            let mut rig = Rig::new("tmuxs-test-kill-warn", &["cat"]);
            rig.app.iterm2_mode = mode;
            rig.ctrl('x');
            let text = rig.screen();
            assert_eq!(text.contains("iTerm2 tabs will close"), warned, "{text}");
            assert!(text.contains("default: No"), "{text}");
        }
    }

    #[test]
    fn create_from_the_query_selects_the_new_session() {
        let mut rig = Rig::new("tmuxs-test-create", &["cat", "dog"]);
        rig.type_str("zebra");
        assert!(rig.app.sessions.is_empty());
        rig.ctrl('n');
        let text = rig.screen();
        assert!(text.contains("Name:      zebra"), "{text}");
        assert!(text.contains("Directory: /tmp"), "{text}");
        rig.press(KeyCode::Enter);
        assert_eq!(rig.names(), ["cat", "dog", "zebra"]);
        assert_eq!(rig.selected(), "zebra");
        assert!(rig.app.popup.is_none());
        assert!(rig.screen().contains("Created"));
        let zebra = rig.server.0.list_sessions().unwrap().pop().unwrap();
        assert_eq!(zebra.path, "/tmp");
    }

    #[test]
    fn create_expands_tilde_and_starts_in_the_chosen_directory() {
        let mut rig = Rig::new("tmuxs-test-create-dir", &[]);
        rig.ctrl('n');
        rig.type_str("home");
        rig.press(KeyCode::Tab);
        rig.backspace(rig.app.launch_dir.len());
        rig.type_str("~");
        rig.press(KeyCode::Enter);
        let sessions = rig.server.0.list_sessions().unwrap();
        let home = std::env::var("HOME").unwrap();
        assert_eq!(sessions[0].path, home);
    }

    #[test]
    fn create_errors_stay_inside_the_popup() {
        let mut rig = Rig::new("tmuxs-test-create-err", &["cat"]);
        rig.ctrl('n');
        rig.press(KeyCode::Enter);
        assert!(rig.screen().contains("The name cannot be empty."));
        rig.type_str("cat");
        rig.press(KeyCode::Enter);
        let text = rig.screen();
        assert!(text.contains("duplicate session: cat"), "{text}");
        rig.backspace(3);
        rig.type_str("new");
        rig.press(KeyCode::Tab);
        rig.type_str("/nope");
        rig.press(KeyCode::Enter);
        let text = rig.screen();
        assert!(
            text.contains("directory does not exist: /tmp/nope"),
            "{text}"
        );
        assert_eq!(rig.names(), ["cat"]);
        rig.backspace(5);
        rig.press(KeyCode::Enter);
        assert_eq!(rig.names(), ["cat", "new"]);
        assert!(rig.app.popup.is_none());
    }

    #[test]
    fn popups_capture_typing_and_esc_quits_only_without_one() {
        let mut rig = Rig::new("tmuxs-test-popup-keys", &["cat"]);
        rig.ctrl('n');
        rig.type_str("abc");
        assert_eq!(rig.app.query, "", "typing went to the Popup");
        rig.ctrl('c');
        assert!(rig.app.popup.is_none() && !rig.app.should_quit);
        rig.press(KeyCode::Esc);
        assert!(rig.app.should_quit);
    }
}
