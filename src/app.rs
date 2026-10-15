//! Application state and key handling. No terminal or tmux access here.

use crate::attach::{NESTED_MESSAGE, is_nested};
use crate::parse::{Session, matches_query};
use crate::popup::{Action, Done, Popup};
use crate::settings::Settings;
use crate::tmux::TmuxError;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::TableState;

#[derive(Debug, Default)]
pub struct App {
    /// Every Session tmux reported.
    pub all_sessions: Vec<Session>,
    /// The Sessions matching the Search query; these are the table rows.
    pub sessions: Vec<Session>,
    pub table_state: TableState,
    /// The text typed into the always-focused search bar.
    pub query: String,
    /// Basename of `$SHELL`, used for Pane state.
    pub shell: Option<String>,
    /// Current time, epoch seconds, for Activity age. Set by the event loop.
    pub now: u64,
    /// Set when tmux cannot be queried (for example tmux is not installed).
    pub error: Option<String>,
    pub should_quit: bool,
    /// Whether Attach uses tmux control mode (`-CC`).
    pub iterm2_mode: bool,
    /// A result message for the Status bar, cleared by the next key press.
    pub message: Option<String>,
    /// The Session id to Attach to; `events::run` takes it and spawns tmux.
    pub attach_request: Option<String>,
    /// The open Popup, if any. While it is open, typing goes to it.
    pub popup: Option<Popup>,
    /// The tmux change a Popup asked for; `events::run_pending` executes it.
    pub pending_action: Option<Action>,
    /// The directory `tmuxs` was launched from, the create Popup's default.
    pub launch_dir: String,
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
            launch_dir: std::env::current_dir()
                .map(|dir| dir.to_string_lossy().into_owned())
                .unwrap_or_default(),
            ..Self::default()
        }
    }

    /// Replaces the Session list, keeping the selection on the same Session id.
    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.all_sessions = sessions;
        self.apply_query();
    }

    /// Recomputes the visible Sessions from the Search query, keeping the
    /// selection on the same Session id when it is still visible.
    fn apply_query(&mut self) {
        let selected_id = self.selected_session().map(|s| s.id.clone());
        let previous_index = self.table_state.selected();
        self.sessions = self
            .all_sessions
            .iter()
            .filter(|s| matches_query(&s.name, &self.query))
            .cloned()
            .collect();
        let index = selected_id
            .and_then(|id| self.sessions.iter().position(|s| s.id == id))
            .or(previous_index)
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
        if self.popup.is_some() {
            self.handle_popup_key(key);
            return;
        }
        match key.code {
            KeyCode::Char('r') if ctrl => self.open_rename(),
            KeyCode::F(2) => self.open_rename(),
            KeyCode::Char('n') if ctrl => {
                self.popup = Some(Popup::create(&self.query, &self.launch_dir));
            }
            KeyCode::Char('x') if ctrl => self.open_kill(),
            KeyCode::Delete => self.open_kill(),
            KeyCode::Enter => self.request_attach(),
            KeyCode::F(3) => self.toggle_iterm2_mode(),
            KeyCode::Char('t') if ctrl => self.toggle_iterm2_mode(),
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if ctrl => self.should_quit = true,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') if ctrl => self.move_selection(-1),
            KeyCode::Char('j') if ctrl => self.move_selection(1),
            KeyCode::Backspace => {
                self.query.pop();
                self.apply_query();
            }
            KeyCode::Char(c) if !ctrl && !key.modifiers.contains(KeyModifiers::ALT) => {
                self.query.push(c);
                self.apply_query();
            }
            _ => {}
        }
    }

    fn handle_popup_key(&mut self, key: KeyEvent) {
        let Some(popup) = &mut self.popup else {
            return;
        };
        if popup.cancels(key) {
            self.popup = None;
        } else {
            self.pending_action = popup.handle_key(key);
        }
    }

    fn open_rename(&mut self) {
        if let Some(s) = self.selected_session() {
            self.popup = Some(Popup::rename(&s.id, &s.name));
        }
    }

    fn open_kill(&mut self) {
        if let Some(s) = self.selected_session() {
            self.popup = Some(Popup::kill(&s.id, &s.name));
        }
    }

    /// Applies the outcome of the pending action, after the list was refreshed:
    /// success closes the Popup, selects the affected Session and sets the
    /// Status bar message; failure keeps the Popup open and shows the error in it.
    pub fn finish_action(&mut self, result: Result<Done, TmuxError>) {
        match result {
            Ok(done) => {
                self.popup = None;
                self.message = Some(done.message.into());
                if let Some(id) = done.select {
                    self.select_id(&id);
                }
            }
            Err(error) => {
                if let Some(popup) = &mut self.popup {
                    popup.set_error(error.to_string());
                }
            }
        }
    }

    /// Selects a Session by id, clearing the Search query if it hides it.
    fn select_id(&mut self, id: &str) {
        if !self.sessions.iter().any(|s| s.id == id) {
            self.query.clear();
            self.apply_query();
        }
        if let Some(index) = self.sessions.iter().position(|s| s.id == id) {
            self.table_state.select(Some(index));
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
            ..Session::default()
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

    fn type_str(app: &mut App, text: &str) {
        for c in text.chars() {
            app.handle_key(key(KeyCode::Char(c), KeyModifiers::NONE));
        }
    }

    fn names(app: &App) -> Vec<&str> {
        app.sessions.iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn typing_narrows_the_table_and_backspace_widens_it() {
        let mut app = app_with(&["cat", "Category", "dog"]);
        type_str(&mut app, "CAT");
        assert_eq!(app.query, "CAT");
        assert_eq!(names(&app), ["cat", "Category"]);
        type_str(&mut app, "x");
        assert!(names(&app).is_empty());
        assert!(app.selected_session().is_none());
        app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(names(&app), ["cat", "Category"]);
        for _ in 0..5 {
            app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        }
        assert_eq!(names(&app), ["cat", "Category", "dog"]);
    }

    #[test]
    fn control_chords_do_not_edit_the_query() {
        let mut app = app_with(&["a"]);
        app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.query, "");
    }

    #[test]
    fn selection_stays_on_the_same_session_while_filtering() {
        let mut app = app_with(&["cat", "category", "dog"]);
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.selected_session().unwrap().name, "category");
        type_str(&mut app, "cat");
        assert_eq!(app.selected_session().unwrap().name, "category");
        type_str(&mut app, "e");
        assert_eq!(app.selected_session().unwrap().name, "category");
        app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        app.handle_key(key(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(app.selected_session().unwrap().name, "category");
    }

    #[test]
    fn refresh_keeps_the_filter_and_the_selection() {
        let mut app = app_with(&["cat", "category", "dog"]);
        type_str(&mut app, "cat");
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        app.set_sessions(vec![
            session("$0", "cat"),
            session("$1", "category"),
            session("$2", "dog"),
            session("$3", "caterpillar"),
        ]);
        assert_eq!(names(&app), ["cat", "category", "caterpillar"]);
        assert_eq!(app.selected_session().unwrap().id, "$1");
    }

    #[test]
    fn real_tmux_search_and_refresh_keep_the_selection_on_the_same_session() {
        use crate::tmux::tests::TestServer;
        let server = TestServer::start("tmuxs-test-search");
        for name in ["category", "dog", "cat"] {
            server.tmux(&["new-session", "-d", "-s", name]);
        }
        let mut app = App::default();
        app.set_result(server.0.list_sessions());
        // Substring semantics: `cat` matches `category` too, but never `dog`.
        type_str(&mut app, "cat");
        assert_eq!(names(&app), ["cat", "category"]);
        app.handle_key(key(KeyCode::Down, KeyModifiers::NONE));
        let category_id = app.selected_session().unwrap().id.clone();
        assert_eq!(app.selected_session().unwrap().name, "category");

        // A new Session sorts before the selected one; selection must not move.
        server.tmux(&["new-session", "-d", "-s", "aaa-cat"]);
        app.set_result(server.0.list_sessions());
        assert_eq!(names(&app), ["aaa-cat", "cat", "category"]);
        assert_eq!(app.selected_session().unwrap().id, category_id);
    }
}
