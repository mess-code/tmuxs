//! The event loop: draw, wait for a key or a tick, refresh the Session list.

use crate::app::App;
use crate::parse;
use crate::terminal::Tui;
use crate::tmux::Tmux;
use crate::ui;
use ratatui::crossterm::event::{self, Event};
use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How often the Session list is re-read when no key is pressed.
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub fn run(terminal: &mut Tui, tmux: &Tmux) -> io::Result<()> {
    let mut app = App {
        shell: std::env::var("SHELL")
            .ok()
            .and_then(|s| parse::shell_basename(&s)),
        ..App::default()
    };
    app.set_result(tmux.list_sessions());
    while !app.should_quit {
        app.now = unix_now();
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

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
