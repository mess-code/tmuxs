# Architecture

`tmuxs` is a single Rust binary crate (edition 2024). Crates: `ratatui` 0.30 (with its re-exported `crossterm`), `clap` (derive), `serde` and `toml` for the settings file, `etcetera` for the config directory.

## Modules (`src/`)

| Module | Role |
|---|---|
| `main.rs` | clap `Args` (`--cc`, `--no-cc`, help text with the environment variables), resolves the initial iTerm2 mode, installs the panic hook, sets up the terminal, runs the event loop, restores the terminal. |
| `app.rs` | `App`: all UI state (Sessions, filtered rows, selection, Search query, message, open Popup, iTerm2 mode, pending action, attach request) and `handle_key`. No terminal or tmux access, so it is unit-testable. |
| `popup.rs` | `Popup` (rename, create, kill), its key handling and `Action` (what to change in tmux). A Popup never talks to tmux. |
| `parse.rs` | Pure functions: parse `list-sessions` / `list-panes` output, Pane state, Window summary, Activity age, substring filter, sorting. |
| `tmux.rs` | `Tmux`: runs `tmux` via `std::process::Command`, honors `TMUXS_SOCKET` (`-L`), lists, creates, renames and kills Sessions by `$id`. `TmuxError` distinguishes "tmux not installed" from other failures. |
| `attach.rs` | Pure `attach_command` (program and arguments) and `is_nested` (`$TMUX` check) with the refusal message. |
| `settings.rs` | iTerm2 mode resolution (`resolve_iterm2_mode`) and load/save of `settings.toml`. |
| `ui.rs` | Rendering with ratatui: search bar, table or empty state or error, Window summary line, Status bar, Popups. |
| `events.rs` | The event loop, running a pending `Action`, and spawn-and-return Attach. |
| `terminal.rs` | Raw mode and alternate screen setup/restore and the panic hook. |

## Data flow

1. `main` reads flags and environment variables, builds `Settings`, `App` and `Tmux`.
2. `events::run` refreshes the list: `tmux list-sessions -F` (tab-separated: id, name, window count, attached count, activity epoch, `session_path`) then `tmux list-panes -a -F` (session id, window index, window name, pane dead flag, foreground command). Panes are attached to Sessions by id. No server (`no server running` or `error connecting to`) means an empty list, not an error. If Pane listing fails, Pane data is just empty.
3. Each loop: set `now`, draw, wait up to 1 second for an event. A key goes to `App::handle_key`. Popup submits become a `pending_action`, which `run_pending` executes (create/rename/kill), then the list is refreshed and the outcome is applied: success closes the Popup and sets the Status bar message; failure keeps the Popup open with the error inside. If no event arrives within 1 second the list is refreshed.
4. Enter sets `attach_request` (or a refusal message when `$TMUX` is set). The loop then runs `attach`.

Tab is used as the field separator because tmux rejects tabs in Session names (verified on tmux 3.8 in the research notes).

## Why every command targets `$id`

See [usage.md](usage.md#workflows). In short: on tmux 3.8 `-t =name` fails for Pane-taking commands, names containing `.` or `:` cannot be targeted by name, and bare names prefix-match. The `session_id` (`$N`) is exact. Selection is also tracked by id, so it follows a Session across refresh, filtering and renames.

## Why the terminal is restored before spawning tmux

`tmuxs` runs in raw mode on the alternate screen with the cursor hidden. tmux (plain or `-CC`) needs a normal terminal: it sets up its own terminal mode and screen, and in `-CC` mode iTerm2 reads its control protocol from that terminal. So `events::attach` calls `terminal::restore()` (raw mode off, alternate screen left, cursor shown), runs tmux with `Command::status()` and waits, then calls `terminal::setup()` again and refreshes. This spawn-and-return design follows `sshs`, which also spawns and waits (it does not `exec`), so detaching from tmux lands you back in the picker.

`terminal::install_panic_hook` wraps the previous panic hook so a panic restores the terminal before the message prints. The panic hook was not exercised automatically; `src/terminal.rs` documents a manual check (add `panic!()` after setup and confirm the shell stays usable). That manual check is part of the [owner checklist](development.md#manual-verification-for-the-owner).

## Error handling

- `tmux` not on `PATH`: `TmuxError::NotInstalled`. The picker stays open and shows a red message in the table area: "tmux was not found on your PATH. Install tmux and try again." The list refresh keeps retrying every second, so it recovers if tmux appears.
- Other tmux list failures show "tmux failed: <message>" in the same place.
- Failed settings save: Status bar "Could not save settings: ...".

## Testing seams

Pure tests on `parse`, `attach`, `settings`; `App` tests with synthetic Sessions; whole-app tests in `events.rs` and `ui.rs` against a real tmux server on a throwaway `tmuxs-test-*` socket with `TestBackend`. See [development.md](development.md).
