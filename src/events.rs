//! The event loop: draw, wait for a key or a tick, refresh the Session list.

use crate::app::App;
use crate::terminal::Tui;
use crate::tmux::Tmux;
use crate::ui;
use ratatui::crossterm::event::{self, Event};
use std::io;
use std::time::Duration;

/// How often the Session list is re-read when no key is pressed.
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub fn run(terminal: &mut Tui, tmux: &Tmux) -> io::Result<()> {
    let mut app = App::default();
    app.set_result(tmux.list_sessions());
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        if event::poll(REFRESH_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        } else {
            app.set_result(tmux.list_sessions());
        }
    }
    Ok(())
}
