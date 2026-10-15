# tmuxs

`tmuxs` is a terminal program that shows every tmux Session in a table and lets you Attach to, create, rename and kill them from the keyboard. Vocabulary (Session, Window, Pane, Pane state, Attach, iTerm2 mode, Search query, Activity age, Popup, Status bar) follows `CONTEXT.md` in the repository root.

## Platform support

- macOS: the target platform. All automated tests were run on macOS with tmux 3.8.
- Linux: should work, because nothing in the code is macOS-specific, but it is unverified.
- Windows: not supported. tmux itself needs WSL there.

## Requirements

- `tmux` on your `PATH`. Behavior was checked against tmux 3.8 only.
- A Rust toolchain to build (ratatui 0.30 needs a recent rustc; the research notes give MSRV 1.88).
- iTerm2 only if you want iTerm2 mode. Plain mode works in any terminal.

## Install

```sh
cargo install --path .
```

No shell profile is edited. Add Cargo's bin directory to your `PATH` yourself if it is not there already.

## Quick start

1. Run `tmuxs`. With no tmux server running you see an empty state: "No tmux sessions. Press Ctrl-N to create one."
2. Press Ctrl-N, type a name, Enter. The new Session is selected.
3. Type part of a name to narrow the table. Press Enter to Attach.
4. Detach from tmux (prefix + d, or Shell > tmux > Detach in iTerm2 mode). You land back in the picker with a refreshed list.

## Pages

- [Usage](usage.md)
- [Key bindings](keybindings.md)
- [iTerm2 mode](iterm2-cc-mode.md)
- [Architecture](architecture.md)
- [Development](development.md)
- [Troubleshooting](troubleshooting.md)

## Verified versus unverified

Every page marks behavior that was not verified as "Unverified". Anything not so marked is backed by the code, the automated tests, or the cited research (`.scratch/tmuxs/research.md`). What was never tried by hand in a real iTerm2 is listed in [development.md](development.md#manual-verification-for-the-owner).
