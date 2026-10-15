//! Application state and key handling. No terminal or tmux access here.

use crate::attach::{NESTED_MESSAGE, is_nested};
use crate::parse::Session;
use crate::settings::Settings;
use crate::tmux::TmuxError;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::TableState;

#[derive(Debug, Default)]
pub struct App {
    pub sessions: Vec<Session>,
    pub table_state: TableState,
    /// Set when tmux cannot be queried (for example tmux is not installed).
    pub error: Option<String>,
    pub should_quit: bool,
    /// Whether Attach uses tmux control mode (`-CC`).
    pub iterm2_mode: bool,
    /// A result message for the Status bar, cleared by the next key press.
    pub message: Option<String>,
    /// The Session id to Attach to; `events::run` takes it and spawns tmux.
    pub attach_request: Option<String>,
    settings: Settings,
    /// The value of `$TMUX`, passed in so the nested check is testable.
    tmux_env: Option<String>,
}

impl App {
    pub fn new(settings: Settings, iterm2_mode: bool, tmux_env: Option<String>) -> Self {
        Self {
            settings,
            iterm2_mode,
            tmux_env,
            ..Self::default()
        }
    }

    /// Replaces the Session list, keeping the selection on the same Session id.
    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        let selected_id = self.selected_session().map(|s| s.id.clone());
        self.sessions = sessions;
        let index = selected_id
            .and_then(|id| self.sessions.iter().position(|s| s.id == id))
            .or_else(|| self.table_state.selected())
            .map(|i| i.min(self.sessions.len().saturating_sub(1)));
        self.table_state.select(if self.sessions.is_empty() {
            None
        } else {
            index.or(Some(0))
        });
    }

    pub fn set_result(&mut self, result: Result<Vec<Session>, TmuxError>) {
        match result {
            Ok(sessions) => {
                self.error = None;
                self.set_sessions(sessions);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub fn selected_session(&self) -> Option<&Session> {
        self.table_state
            .selected()
            .and_then(|i| self.sessions.get(i))
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        self.message = None;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Enter => self.request_attach(),
            KeyCode::F(3) => self.toggle_iterm2_mode(),
            KeyCode::Char('t') if ctrl => self.toggle_iterm2_mode(),
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if ctrl => self.should_quit = true,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') if ctrl => self.move_selection(-1),
            KeyCode::Char('j') if ctrl => self.move_selection(1),
            _ => {}
        }
    }

    fn request_attach(&mut self) {
        let Some(id) = self.selected_session().map(|s| s.id.clone()) else {
            return;
        };
        if is_nested(self.tmux_env.as_deref()) {
            self.message = Some(NESTED_MESSAGE.into());
        } else {
            self.attach_request = Some(id);
        }
    }

    fn toggle_iterm2_mode(&mut self) {
        self.iterm2_mode = !self.iterm2_mode;
        if let Err(error) = self.settings.save_iterm2_mode(self.iterm2_mode) {
            self.message = Some(format!("Could not save settings: {error}"));
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.sessions.is_empty() {
            return;
        }
        let last = self.sessions.len() - 1;
        let current = self.table_state.selected().unwrap_or(0);
        self.table_state
            .select(Some(current.saturating_add_signed(delta).min(last)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str, name: &str) -> Session {
        Session {
            id: id.into(),
            name: name.into(),
            windows: 1,
            attached: 0,
        }
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn app_with(names: &[&str]) -> App {
        let mut app = App::default();
        app.set_sessions(
            names
                .iter()
                .enumerate()
                .map(|(i, n)| session(&format!("${i}"), n))
                .collect(),
        );
        app
    }

    #[test]
    fn first_session_is_selected_initially() {
        assert_eq!(app_with(&["a", "b"]).selected_session().unwrap().name, "a");
        assert!(app_with(&[]).selected_session().is_none());
    }

    #[test]
    fn arrows_and_ctrl_jk_move_and_clamp() {
        let mut app = app_with(&["a", "b", "c"]);
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Char('j'), KeyModifiers::CONTROL));
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.selected_session().unwrap().name, "c");
        app.handle_key(key(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(app.selected_session().unwrap().name, "b");
        app.handle_key(key(KeyCode::Up, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.selected_session().unwrap().name, "a");
    }

    #[test]
    fn esc_and_ctrl_c_quit_other_keys_do_not() {
        for k in [
            key(KeyCode::Esc, KeyModifiers::NONE),
            key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            let mut app = app_with(&["a"]);
            app.handle_key(k);
            assert!(app.should_quit);
        }
        let mut app = app_with(&["a"]);
        app.handle_key(key(KeyCode::Char('c'), KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert!(!app.should_quit);
    }

    #[test]
    fn release_events_are_ignored() {
        let mut app = app_with(&["a"]);
        let mut release = key(KeyCode::Esc, KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        app.handle_key(release);
        assert!(!app.should_quit);
    }

    #[test]
    fn selection_follows_the_session_id_across_refresh() {
        let mut app = app_with(&["a", "b"]);
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        app.set_sessions(vec![
            session("$7", "0"),
            session("$0", "a"),
            session("$1", "b"),
        ]);
        assert_eq!(app.selected_session().unwrap().id, "$1");
    }

    #[test]
    fn selection_is_clamped_when_the_session_disappears() {
        let mut app = app_with(&["a", "b", "c"]);
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        app.set_sessions(vec![session("$0", "a")]);
        assert_eq!(app.selected_session().unwrap().id, "$0");
        app.set_sessions(vec![]);
        assert!(app.selected_session().is_none());
    }

    fn app_in(dir: &std::path::Path, tmux_env: Option<&str>) -> App {
        let mut app = App::new(
            Settings::new(dir.to_str()),
            false,
            tmux_env.map(String::from),
        );
        app.set_sessions(vec![session("$4", "a")]);
        app
    }

    #[test]
    fn enter_requests_attach_to_the_selected_session_id() {
        let mut app = app_in(&std::env::temp_dir(), None);
        app.handle_key(key(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.attach_request.as_deref(), Some("$4"));
        let mut empty = App::default();
        empty.handle_key(key(KeyCode::Enter, KeyModifiers::NONE));
        assert!(empty.attach_request.is_none());
    }

    #[test]
    fn enter_is_refused_inside_tmux_in_both_modes() {
        for mode in [false, true] {
            let mut app = app_in(&std::env::temp_dir(), Some("/tmp/tmux-1/default,1,0"));
            app.iterm2_mode = mode;
            app.handle_key(key(KeyCode::Enter, KeyModifiers::NONE));
            assert!(app.attach_request.is_none());
            assert_eq!(app.message.as_deref(), Some(NESTED_MESSAGE));
            app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
            assert!(app.message.is_none());
        }
    }

    #[test]
    fn ctrl_t_and_f3_toggle_and_save_to_the_settings_dir() {
        let dir = std::env::temp_dir().join("tmuxs-test-toggle");
        let _ = std::fs::remove_dir_all(&dir);
        let mut app = app_in(&dir, None);
        let saved = || Settings::new(dir.to_str()).load_iterm2_mode();
        app.handle_key(key(KeyCode::Char('t'), KeyModifiers::CONTROL));
        assert!(app.iterm2_mode);
        assert_eq!(saved(), Some(true));
        app.handle_key(key(KeyCode::F(3), KeyModifiers::NONE));
        assert!(!app.iterm2_mode);
        assert_eq!(saved(), Some(false));
        assert!(app.message.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_save_shows_a_message_and_keeps_the_toggle() {
        let blocker = std::env::temp_dir().join("tmuxs-test-toggle-blocker");
        std::fs::write(&blocker, "").unwrap();
        let mut app = app_in(&blocker, None);
        app.handle_key(key(KeyCode::F(3), KeyModifiers::NONE));
        assert!(app.iterm2_mode);
        assert!(app.message.unwrap().starts_with("Could not save settings"));
        let _ = std::fs::remove_file(&blocker);
    }

    #[test]
    fn errors_are_kept_and_cleared() {
        let mut app = app_with(&["a"]);
        app.set_result(Err(TmuxError::NotInstalled));
        assert!(app.error.is_some());
        app.set_result(Ok(vec![]));
        assert!(app.error.is_none());
    }
}
