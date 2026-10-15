//! Terminal setup and restore, plus the panic hook that restores the terminal.

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::cursor::{Hide, Show};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::{self, Stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Enters raw mode and the alternate screen and hides the cursor.
pub fn setup() -> io::Result<Tui> {
    enable_raw_mode()?;
    if let Err(error) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
        let _ = restore();
        return Err(error);
    }
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

/// Undoes `setup`. Safe to call more than once and when setup only half worked.
pub fn restore() -> io::Result<()> {
    let raw = disable_raw_mode();
    let screen = execute!(io::stdout(), Show, LeaveAlternateScreen);
    raw.and(screen)
}

/// Makes any panic restore the terminal before the panic message is printed.
///
/// Manual check: add `panic!()` right after `terminal::setup()` and run in a
/// terminal; the shell must stay usable and the message must be readable.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore();
        previous(info);
    }));
}
