# tmuxs

A small terminal picker for tmux Sessions, in the spirit of `sshs` for SSH hosts. It lists every Session in a table (Windows, Panes with busy/idle/dead counts, attached state, Activity age, directory) and lets you Attach to, create, rename and kill Sessions with single key chords. A toggle chooses between iTerm2's tmux control mode (`tmux -CC`) and a plain tmux client.

Platform: macOS is the target and the only tested platform. Linux should work but is unverified. Windows is not supported (tmux needs WSL).

## Install

```sh
cargo install --path .
```

This builds and copies the binary to Cargo's bin directory. It does not touch your shell profile; make sure that directory is on your `PATH` yourself. Requires `tmux` on your `PATH` and a Rust toolchain (the code uses edition 2024 and ratatui 0.30).

## Quick start

```sh
tmuxs            # open the picker
tmuxs --no-cc    # force a plain tmux client for this run
tmuxs --cc       # force iTerm2 mode (tmux -CC) for this run
```

Type to search, Up/Down to move, Enter to Attach, Ctrl-N to create, Ctrl-R to rename, Ctrl-X to kill, Ctrl-T to toggle iTerm2 mode, Esc to quit.

## Documentation

- [docs/index.md](docs/index.md): overview, install, quick start
- [docs/usage.md](docs/usage.md): workflows, flags, environment variables, settings
- [docs/keybindings.md](docs/keybindings.md): every key binding
- [docs/iterm2-cc-mode.md](docs/iterm2-cc-mode.md): iTerm2 mode and `-CC`
- [docs/architecture.md](docs/architecture.md): modules and data flow
- [docs/development.md](docs/development.md): build, test, lint, cross-check checklist
- [docs/troubleshooting.md](docs/troubleshooting.md): common problems
