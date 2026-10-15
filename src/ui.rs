//! Rendering. Reads `App`, never mutates anything except the table scroll state.

use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Cell, HighlightSpacing, Paragraph, Row, Table};

const HINTS: &str = "(Esc) quit | (↑/↓, ctrl+k/ctrl+j) move";
const EMPTY_HINT: &str = "No tmux sessions. Press Ctrl-N to create one.";

pub fn render(frame: &mut Frame, app: &mut App) {
    let [table_area, status_area] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(frame.area());

    if let Some(error) = &app.error {
        frame.render_widget(
            Paragraph::new(error.as_str())
                .style(Style::new().fg(Color::Red))
                .block(Block::bordered()),
            table_area,
        );
    } else if app.sessions.is_empty() {
        frame.render_widget(
            Paragraph::new(EMPTY_HINT)
                .centered()
                .block(Block::bordered().title(" tmuxs ")),
            table_area,
        );
    } else {
        render_table(frame, app, table_area);
    }

    frame.render_widget(Paragraph::new(HINTS).centered(), status_area);
}

fn render_table(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let header = Row::new(["Name", "Windows", "State"])
        .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let rows = app.sessions.iter().map(|s| {
        let (label, color) = if s.is_attached() {
            ("attached", Color::Green)
        } else {
            ("detached", Color::Yellow)
        };
        Row::new([
            Cell::from(s.name.as_str()),
            Cell::from(s.windows.to_string()),
            Cell::from(label).style(Style::new().fg(color)),
        ])
    });
    let widths = [
        Constraint::Min(10),
        Constraint::Length(8),
        Constraint::Length(10),
    ];
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
        let mut terminal = Terminal::new(TestBackend::new(60, 8)).unwrap();
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
}
