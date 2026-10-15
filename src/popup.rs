//! Popup state and key handling for rename, create and kill confirmation.
//!
//! A Popup never talks to tmux. Submitting it yields an `Action`; the event
//! loop runs that action and either closes the Popup or shows the error in it.

use crate::tmux::{Tmux, TmuxError};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const EMPTY_NAME: &str = "The name cannot be empty.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Name,
    Dir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popup {
    Rename {
        id: String,
        input: String,
        error: Option<String>,
    },
    Create {
        name: String,
        dir: String,
        field: Field,
        error: Option<String>,
    },
    Kill {
        id: String,
        name: String,
        error: Option<String>,
    },
}

/// A tmux change the user asked for. Every Session is targeted by its `$id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Rename { id: String, name: String },
    Create { name: String, dir: String },
    Kill { id: String },
}

/// What a successful `Action` leaves behind.
#[derive(Debug, PartialEq, Eq)]
pub struct Done {
    /// The Status bar message.
    pub message: &'static str,
    /// The Session to select afterwards, if any.
    pub select: Option<String>,
}

impl Action {
    pub fn run(&self, tmux: &Tmux) -> Result<Done, TmuxError> {
        match self {
            Action::Rename { id, name } => {
                tmux.rename_session(id, name)?;
                Ok(Done {
                    message: "Renamed",
                    select: Some(id.clone()),
                })
            }
            Action::Create { name, dir } => Ok(Done {
                message: "Created",
                select: Some(tmux.create_session(name, dir)?),
            }),
            Action::Kill { id } => {
                tmux.kill_session(id)?;
                Ok(Done {
                    message: "Killed",
                    select: None,
                })
            }
        }
    }
}

impl Popup {
    pub fn rename(id: &str, current_name: &str) -> Self {
        Popup::Rename {
            id: id.into(),
            input: current_name.into(),
            error: None,
        }
    }

    pub fn create(query: &str, launch_dir: &str) -> Self {
        Popup::Create {
            name: query.into(),
            dir: launch_dir.into(),
            field: Field::Name,
            error: None,
        }
    }

    pub fn kill(id: &str, name: &str) -> Self {
        Popup::Kill {
            id: id.into(),
            name: name.into(),
            error: None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            Popup::Rename { error, .. }
            | Popup::Create { error, .. }
            | Popup::Kill { error, .. } => error.as_deref(),
        }
    }

    pub fn set_error(&mut self, message: String) {
        match self {
            Popup::Rename { error, .. }
            | Popup::Create { error, .. }
            | Popup::Kill { error, .. } => *error = Some(message),
        }
    }

    /// Whether the key closes the Popup without doing anything: Esc and Ctrl-C
    /// always, and Enter or `n` in the kill confirmation (the default is No).
    pub fn cancels(&self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => true,
            KeyCode::Char('c') => ctrl,
            KeyCode::Enter | KeyCode::Char('n' | 'N') => matches!(self, Popup::Kill { .. }),
            _ => false,
        }
    }

    /// Handles a key while the Popup is open. Closing keys (see `cancels`) are left
    /// to the caller. Returns an `Action` when the user confirmed the Popup.
    ///
    /// Tab, Shift-Tab, Up and Down switch fields in the create Popup.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        match self {
            Popup::Kill { id, .. } => match key.code {
                KeyCode::Char('y' | 'Y') => Some(Action::Kill { id: id.clone() }),
                _ => None,
            },
            Popup::Rename { id, input, error } => {
                if key.code == KeyCode::Enter {
                    let name = input.trim();
                    if name.is_empty() {
                        *error = Some(EMPTY_NAME.into());
                        return None;
                    }
                    return Some(Action::Rename {
                        id: id.clone(),
                        name: name.into(),
                    });
                }
                edit(input, error, key);
                None
            }
            Popup::Create {
                name,
                dir,
                field,
                error,
            } => match key.code {
                KeyCode::Enter => {
                    let trimmed = name.trim();
                    if trimmed.is_empty() {
                        *error = Some(EMPTY_NAME.into());
                        *field = Field::Name;
                        return None;
                    }
                    Some(Action::Create {
                        name: trimmed.into(),
                        dir: dir.trim().into(),
                    })
                }
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                    *field = match field {
                        Field::Name => Field::Dir,
                        Field::Dir => Field::Name,
                    };
                    None
                }
                _ => {
                    match field {
                        Field::Name => edit(name, error, key),
                        Field::Dir => edit(dir, error, key),
                    }
                    None
                }
            },
        }
    }
}

/// Types into or deletes from the end of a text field, clearing the error.
fn edit(text: &mut String, error: &mut Option<String>, key: KeyEvent) {
    match key.code {
        KeyCode::Backspace => {
            text.pop();
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            text.push(c);
        }
        _ => return,
    }
    *error = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(popup: &mut Popup, code: KeyCode) -> Option<Action> {
        popup.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn type_str(popup: &mut Popup, text: &str) {
        for c in text.chars() {
            press(popup, KeyCode::Char(c));
        }
    }

    #[test]
    fn rename_is_prefilled_edited_and_submitted_by_id() {
        let mut popup = Popup::rename("$3", "cat");
        press(&mut popup, KeyCode::Backspace);
        type_str(&mut popup, "pet");
        assert_eq!(
            press(&mut popup, KeyCode::Enter),
            Some(Action::Rename {
                id: "$3".into(),
                name: "capet".into()
            })
        );
    }

    #[test]
    fn an_empty_name_is_rejected_inside_the_popup() {
        let mut popup = Popup::rename("$3", "a");
        press(&mut popup, KeyCode::Backspace);
        assert_eq!(press(&mut popup, KeyCode::Enter), None);
        assert_eq!(popup.error(), Some(EMPTY_NAME));
        type_str(&mut popup, " ");
        assert_eq!(popup.error(), None);
        assert_eq!(press(&mut popup, KeyCode::Enter), None);
        assert!(popup.error().is_some());

        let mut create = Popup::create("  ", "/tmp");
        assert_eq!(press(&mut create, KeyCode::Enter), None);
        assert_eq!(create.error(), Some(EMPTY_NAME));
    }

    #[test]
    fn create_prefills_the_name_and_directory_and_tab_switches_fields() {
        let mut popup = Popup::create("feat", "/work");
        type_str(&mut popup, "x");
        press(&mut popup, KeyCode::Tab);
        type_str(&mut popup, "/sub");
        press(&mut popup, KeyCode::Down);
        type_str(&mut popup, "y");
        press(&mut popup, KeyCode::Up);
        type_str(&mut popup, "z");
        press(&mut popup, KeyCode::BackTab);
        assert_eq!(
            press(&mut popup, KeyCode::Enter),
            Some(Action::Create {
                name: "featxy".into(),
                dir: "/work/subz".into()
            })
        );
    }

    #[test]
    fn kill_needs_an_explicit_y_and_everything_else_does_nothing() {
        for code in [KeyCode::Enter, KeyCode::Char('n'), KeyCode::Char('x')] {
            assert_eq!(press(&mut Popup::kill("$1", "a"), code), None);
        }
        assert_eq!(
            press(&mut Popup::kill("$1", "a"), KeyCode::Char('y')),
            Some(Action::Kill { id: "$1".into() })
        );
    }

    #[test]
    fn cancel_keys() {
        let k = |code| KeyEvent::new(code, KeyModifiers::NONE);
        let rename = Popup::rename("$1", "a");
        assert!(rename.cancels(k(KeyCode::Esc)));
        assert!(rename.cancels(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!rename.cancels(k(KeyCode::Char('c'))));
        assert!(!rename.cancels(k(KeyCode::Enter)) && !rename.cancels(k(KeyCode::Char('n'))));
        let kill = Popup::kill("$1", "a");
        assert!(kill.cancels(k(KeyCode::Enter)) && kill.cancels(k(KeyCode::Char('N'))));
        assert!(!kill.cancels(k(KeyCode::Char('y'))));
    }

    #[test]
    fn control_chords_do_not_type_into_fields() {
        let mut popup = Popup::rename("$1", "a");
        popup.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        assert_eq!(popup, Popup::rename("$1", "a"));
    }
}
