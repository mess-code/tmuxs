//! Rendering. Reads `App`, never mutates anything except the table scroll state.

use crate::app::App;
use crate::parse::{Session, format_age, window_summary};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Cell, HighlightSpacing, Paragraph, Row, Table};

const HINTS: &str = "(Esc) quit | (↑/↓, ctrl+k/ctrl+j) move | type to search";
const EMPTY_HINT: &str = "No tmux sessions. Press Ctrl-N to create one.";
const NO_MATCH_HINT: &str = "No sessions match the search.";

pub fn render(frame: &mut Frame, app: &mut App) {
    let [search_area, table_area, summary_area, status_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_search_bar(frame, app, search_area);

    if let Some(error) = &app.error {
        frame.render_widget(
            Paragraph::new(error.as_str())
                .style(Style::new().fg(Color::Red))
                .block(Block::bordered()),
            table_area,
        );
    } else if app.sessions.is_empty() {
        let hint = if app.all_sessions.is_empty() {
            EMPTY_HINT
        } else {
            NO_MATCH_HINT
        };
        frame.render_widget(
            Paragraph::new(hint)
                .centered()
                .block(Block::bordered().title(" tmuxs ")),
            table_area,
        );
    } else {
        render_table(frame, app, table_area);
    }

    let summary = app.selected_session().map(window_summary);
    frame.render_widget(
        Paragraph::new(summary.unwrap_or_default()).style(Style::new().fg(Color::Cyan)),
        summary_area,
    );
    frame.render_widget(Paragraph::new(HINTS).centered(), status_area);
}

fn render_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(
        Paragraph::new(app.query.as_str()).block(Block::bordered().title(" Search ")),
        area,
    );
    let typed = app.query.chars().count() as u16;
    frame.set_cursor_position(Position::new(area.x + 1 + typed, area.y + 1));
}

/// `3 (2 busy · 1 idle)`: total Panes, then the non-zero counts per Pane state.
fn pane_summary(session: &Session, shell: Option<&str>) -> String {
    let counts = session.pane_counts(shell);
    let parts: Vec<String> = [
        (counts.busy, "busy"),
        (counts.idle, "idle"),
        (counts.dead, "dead"),
    ]
    .iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, label)| format!("{n} {label}"))
    .collect();
    if parts.is_empty() {
        session.panes.len().to_string()
    } else {
        format!("{} ({})", session.panes.len(), parts.join(" · "))
    }
}

fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    // The Directory column is dropped when no Session reports a working
    // directory, which is how a tmux without `session_path` shows up.
    let show_path = app.sessions.iter().any(|s| !s.path.is_empty());
    let mut titles = vec!["Name", "Windows", "Panes", "State", "Activity"];
    let mut widths = vec![
        Constraint::Min(10),
        Constraint::Length(8),
        Constraint::Length(30),
        Constraint::Length(10),
        Constraint::Length(8),
    ];
    if show_path {
        titles.push("Directory");
        widths.push(Constraint::Min(10));
    }
    let header = Row::new(titles).style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let rows = app.sessions.iter().map(|s| {
        let (label, color) = if s.is_attached() {
            ("attached", Color::Green)
        } else {
            ("detached", Color::Yellow)
        };
        let mut cells = vec![
            Cell::from(s.name.as_str()),
            Cell::from(s.windows.to_string()),
            Cell::from(pane_summary(s, app.shell.as_deref())),
            Cell::from(label).style(Style::new().fg(color)),
            Cell::from(format_age(app.now, s.activity)),
        ];
        if show_path {
            cells.push(Cell::from(s.path.as_str()));
        }
        Row::new(cells)
    });
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::bordered().title(" tmuxs "))
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(" █ ")
        .highlight_spacing(HighlightSpacing::Always);
    frame.render_stateful_widget(table, area, &mut app.table_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Session;
    use crate::tmux::TmuxError;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn draw(app: &mut App) -> (Terminal<TestBackend>, String) {
        let mut terminal = Terminal::new(TestBackend::new(110, 14)).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        let buffer = terminal.backend().buffer();
        let text = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        (terminal, text)
    }

    fn session(id: &str, name: &str, attached: u32) -> Session {
        Session {
            id: id.into(),
            name: name.into(),
            windows: 3,
            attached,
            ..Session::default()
        }
    }

    #[test]
    fn list_state_shows_header_rows_and_states() {
        let mut app = App::default();
        app.set_sessions(vec![session("$1", "cat", 1), session("$2", "category", 0)]);
        let (terminal, text) = draw(&mut app);
        for expected in [
            "Name", "Windows", "State", "cat ", "category", "attached", "detached",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }
        assert!(!text.contains("Ctrl-N"));
        // The selected row is the first one and is highlighted.
        let buffer = terminal.backend().buffer();
        let row = text.lines().position(|l| l.contains("cat ")).unwrap() as u16;
        assert!(buffer[(5, row)].modifier.contains(Modifier::REVERSED));
        let other = text.lines().position(|l| l.contains("category")).unwrap() as u16;
        assert!(!buffer[(5, other)].modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn state_column_is_colored() {
        let mut app = App::default();
        app.set_sessions(vec![session("$1", "a", 1), session("$2", "b", 0)]);
        let (terminal, text) = draw(&mut app);
        let buffer = terminal.backend().buffer();
        let color_of = |needle: &str| {
            let y = text.lines().position(|l| l.contains(needle)).unwrap() as u16;
            let x = text.lines().nth(y as usize).unwrap().find(needle).unwrap();
            let x = text.lines().nth(y as usize).unwrap()[..x].chars().count() as u16;
            buffer[(x, y)].fg
        };
        assert_eq!(color_of("attached"), Color::Green);
        assert_eq!(color_of("detached"), Color::Yellow);
    }

    #[test]
    fn empty_state_points_to_ctrl_n() {
        let (_, text) = draw(&mut App::default());
        assert!(
            text.contains("No tmux sessions. Press Ctrl-N to create one."),
            "{text}"
        );
        assert!(!text.contains("Windows"));
    }

    #[test]
    fn tmux_not_installed_shows_a_plain_message() {
        let mut app = App::default();
        app.set_result(Err(TmuxError::NotInstalled));
        let (_, text) = draw(&mut app);
        assert!(text.contains("tmux was not found on your PATH"), "{text}");
    }

    #[test]
    fn status_bar_shows_key_hints() {
        let (_, text) = draw(&mut App::default());
        assert!(text.contains("(Esc) quit"));
    }

    fn full_session(id: &str, name: &str, activity: u64, path: &str) -> Session {
        let pane = |command: &str, dead: bool, index: u32, window: &str| crate::parse::Pane {
            session_id: id.into(),
            window_index: index,
            window_name: window.into(),
            dead,
            command: command.into(),
        };
        Session {
            activity,
            path: path.into(),
            panes: vec![
                pane("zsh", false, 0, "zsh"),
                pane("claude", false, 1, "claude"),
                pane("node", false, 1, "claude"),
                pane("zsh", true, 2, "old"),
            ],
            ..session(id, name, 0)
        }
    }

    #[test]
    fn table_shows_panes_activity_directory_and_window_summary() {
        let mut app = App {
            now: 10_000,
            ..App::default()
        };
        app.set_sessions(vec![full_session("$1", "work", 10_000 - 12 * 60, "/tmp/x")]);
        let (_, text) = draw(&mut app);
        for expected in [
            "Panes",
            "Activity",
            "Directory",
            "4 (2 busy · 1 idle · 1 dead)",
            "12m",
            "/tmp/x",
            "0:zsh 1:claude 2:old",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }
    }

    #[test]
    fn directory_column_is_dropped_when_tmux_reports_no_path() {
        let mut app = App::default();
        app.set_sessions(vec![full_session("$1", "work", 0, "")]);
        let (_, text) = draw(&mut app);
        assert!(text.contains("Activity"));
        assert!(!text.contains("Directory"), "{text}");
    }

    #[test]
    fn search_bar_shows_the_query_and_the_filtered_rows() {
        let mut app = App::default();
        app.set_sessions(vec![session("$1", "cat", 0), session("$2", "dog", 0)]);
        for c in "cat".chars() {
            app.handle_key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char(c),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        }
        let (mut terminal, text) = draw(&mut app);
        assert!(text.lines().next().unwrap().contains("Search"), "{text}");
        assert!(text.lines().nth(1).unwrap().contains("cat"), "{text}");
        assert!(text.contains("cat "));
        assert!(!text.contains("dog"));
        terminal.backend_mut().assert_cursor_position((4, 1));
    }

    #[test]
    fn a_query_without_matches_says_so_instead_of_pointing_to_ctrl_n() {
        let mut app = App::default();
        app.set_sessions(vec![session("$1", "cat", 0)]);
        app.query = "zzz".into();
        app.handle_key(ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Backspace,
            ratatui::crossterm::event::KeyModifiers::NONE,
        ));
        app.handle_key(ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('z'),
            ratatui::crossterm::event::KeyModifiers::NONE,
        ));
        let (_, text) = draw(&mut app);
        assert!(text.contains("No sessions match the search."), "{text}");
        assert!(!text.contains("Ctrl-N"));
    }

    #[test]
    fn status_bar_hints_mention_typing_to_search() {
        let (_, text) = draw(&mut App::default());
        assert!(text.contains("type to search"));
    }
}
