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
