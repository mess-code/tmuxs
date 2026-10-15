# tmuxs

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: macOS](https://img.shields.io/badge/platform-macOS-lightgrey)
![Rust: edition 2024](https://img.shields.io/badge/rust-edition%202024-orange)

A small terminal picker for tmux Sessions, in the spirit of [`sshs`](https://github.com/quantumsheep/sshs) for SSH hosts. Type to find a Session, press Enter to attach, and come back to the picker when you detach.

I built it to keep long-running agent work in tmux without any other GUI tool, used together with iTerm2.

![tmuxs demo](docs/images/demo.gif)

## Features

- **See everything at a glance.** A table with each Session's Windows, Panes (busy, idle and dead counts), attached state, Activity age and directory.
- **Find fast.** The search bar is always focused: just type to narrow the list by name.
- **Manage Sessions.** Attach, create, rename and kill with single key chords. Kill asks for confirmation, and errors from tmux show up inside the dialog so you can fix your input.
- **iTerm2 mode toggle.** Switch Attach between iTerm2's tmux control mode (`tmux -CC`, native tabs) and a plain tmux client with one key. The choice is remembered.
- **Safe by construction.** Every command targets a Session by its id, so `cat` can never hit `category`, and names with dots or colons work.
- **Returns to the picker.** After you detach, you land back in the list instead of at a shell prompt.

## Requirements

- macOS (the target and the only tested platform; Linux should work but is unverified; Windows is not supported, tmux needs WSL)
- `tmux` on your `PATH` (developed against tmux 3.8)
- A Rust toolchain to build it (edition 2024)
- iTerm2 only if you want the `-CC` mode

## Install

```sh
git clone https://github.com/mess-code/tmuxs
cd tmuxs
cargo install --path .
```

Or straight from GitHub:

```sh
cargo install --git https://github.com/mess-code/tmuxs
```

Cargo puts the binary in `~/.cargo/bin`. Make sure that directory is on your `PATH`; the installer does not edit your shell profile.

## Quick start

```sh
tmuxs            # open the picker
tmuxs --no-cc    # force a plain tmux client for this run
tmuxs --cc       # force iTerm2 mode (tmux -CC) for this run
```

| Key | Action |
|---|---|
| Type | Search Sessions by name |
| Up / Down, Ctrl-K / Ctrl-J | Move the selection |
| Enter | Attach to the selected Session |
| Ctrl-N | Create a Session (name prefilled from your search) |
| Ctrl-R or F2 | Rename |
| Ctrl-X or Delete | Kill, after a y/N confirmation |
| Ctrl-T or F3 | Toggle iTerm2 mode |
| Esc or Ctrl-C | Close a dialog, or quit |

Attach is refused when you are already inside tmux; detach first. The full list of keys, including dialog keys, is in [docs/keybindings.md](docs/keybindings.md).

## Configuration

The only saved setting is the iTerm2 mode, stored in `~/.config/tmuxs/settings.toml`. At startup it comes from, in order: `--cc` / `--no-cc`, the `TMUXS_CC` environment variable (`1` or `0`), the saved value, then automatic (ON inside iTerm2, OFF elsewhere). `TMUXS_CONFIG_DIR` changes the settings directory and `TMUXS_SOCKET` makes every tmux command run as `tmux -L <value>`. See [docs/usage.md](docs/usage.md).

## Documentation

- [docs/index.md](docs/index.md): overview, install, quick start
- [docs/usage.md](docs/usage.md): workflows, flags, environment variables, settings
- [docs/keybindings.md](docs/keybindings.md): every key binding
- [docs/iterm2-cc-mode.md](docs/iterm2-cc-mode.md): iTerm2 mode and `-CC`, including what is not verified
- [docs/architecture.md](docs/architecture.md): modules and data flow
- [docs/development.md](docs/development.md): build, test, lint, safe testing
- [docs/troubleshooting.md](docs/troubleshooting.md): common problems

## Development

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Tests that need tmux use throwaway sockets and never touch your real Sessions or settings. Details are in [docs/development.md](docs/development.md).

## Acknowledgements

The look and feel follow [`sshs`](https://github.com/quantumsheep/sshs), the terminal picker for SSH hosts. No `sshs` code was copied.

## License

[MIT](LICENSE)
