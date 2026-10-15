//! tmuxs: a terminal picker for tmux Sessions.

mod app;
mod events;
mod parse;
mod terminal;
mod tmux;
mod ui;

use clap::Parser;

/// A terminal picker for tmux Sessions.
///
/// Environment: TMUXS_SOCKET makes every tmux command run as `tmux -L <value>`.
#[derive(Parser)]
#[command(version, about)]
struct Args {}

fn main() -> std::io::Result<()> {
    Args::parse();
    terminal::install_panic_hook();
    let mut tui = terminal::setup()?;
    let result = events::run(&mut tui, &tmux::Tmux::from_env());
    terminal::restore()?;
    result
}
