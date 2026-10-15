//! tmuxs: a terminal picker for tmux Sessions.

mod app;
mod attach;
mod events;
mod parse;
mod popup;
mod settings;
mod terminal;
mod tmux;
mod ui;

use app::App;
use clap::Parser;
use settings::{Settings, resolve_iterm2_mode};

/// A terminal picker for tmux Sessions.
///
/// Environment variables:
///
/// TMUXS_SOCKET     run every tmux command as `tmux -L <value>`.
///
/// TMUXS_CC         iTerm2 mode at startup: 1 is ON, 0 is OFF. Overridden by
/// --cc and --no-cc, and overrides the saved choice.
///
/// TMUXS_CONFIG_DIR directory holding settings.toml (default ~/.config/tmuxs).
///
/// Without any of these, iTerm2 mode is the saved choice, or ON when
/// TERM_PROGRAM is iTerm.app and OFF otherwise. Ctrl-T or F3 toggles it.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Attach with iTerm2 mode ON (tmux control mode, `tmux -CC`).
    #[arg(long, conflicts_with = "no_cc")]
    cc: bool,
    /// Attach with iTerm2 mode OFF (a plain tmux client).
    #[arg(long)]
    no_cc: bool,
}

impl Args {
    fn cc_flag(&self) -> Option<bool> {
        match (self.cc, self.no_cc) {
            (true, _) => Some(true),
            (_, true) => Some(false),
            _ => None,
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let settings = Settings::from_env();
    let iterm2_mode = resolve_iterm2_mode(
        args.cc_flag(),
        std::env::var(settings::CC_ENV).ok().as_deref(),
        settings.load_iterm2_mode(),
        std::env::var("TERM_PROGRAM").ok().as_deref(),
    );
    let app = App::new(settings, iterm2_mode, std::env::var("TMUX").ok());
    terminal::install_panic_hook();
    let mut tui = terminal::setup()?;
    let result = events::run(&mut tui, &tmux::Tmux::from_env(), app);
    terminal::restore()?;
    result
}
