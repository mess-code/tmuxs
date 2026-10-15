# Development

## Build, lint, test

```sh
cargo build --release              # binary at target/release/tmuxs
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All four must pass before a change is done. Install with `cargo install --path .` (this does not edit shell profiles).

## Safe testing

Tests and manual probes must never touch your real tmux server or real settings. Use the two variables:

- `TMUXS_SOCKET=tmuxs-test-<name>`: every tmux command becomes `tmux -L tmuxs-test-<name>`, a separate server. Kill it afterwards with `tmux -L tmuxs-test-<name> kill-server`.
- `TMUXS_CONFIG_DIR=$(mktemp -d)`: settings are read and written there.

Example manual run:

```sh
TMUXS_SOCKET=tmuxs-test-manual TMUXS_CONFIG_DIR=$(mktemp -d) cargo run
tmux -L tmuxs-test-manual kill-server
```

The automated tests follow this rule: whole-app tests start a real tmux server on a `tmuxs-test-*` socket (a helper kills it when the test ends), use synthetic key events and `ratatui`'s `TestBackend`, and assert both on the rendered buffer and on real tmux state. Settings tests use directories under the system temp dir. Each test that needs a server uses its own socket name so tests can run in parallel. They need `tmux` on the `PATH`.

At the time of writing `cargo test` runs 81 tests (parsing, Pane state, Activity age, filter, settings, attach command, app keys, Popups, rendering, and real-tmux flows).

## Layout

See [architecture.md](architecture.md).

## Cross-check: keys, flags and environment variables

Each item was verified by reading the code at the location given and, for flags, by running the binary (`cargo run -- --help`, and `cargo run -- --cc --no-cc`, which exits with the clap conflict error). Line numbers refer to the commit that introduced this page.

### Flags

| Item | Code | Verified by |
|---|---|---|
| `--cc` | `src/main.rs` `Args::cc`, `cc_flag()` | `--help` output; code |
| `--no-cc` | `src/main.rs` `Args::no_cc` | `--help` output; code |
| `--cc` with `--no-cc` conflicts | `src/main.rs` `conflicts_with` | ran the binary: clap error |
| `-h/--help`, `-V/--version` | clap `#[command(version, about)]` | ran the binary |

### Environment variables

| Item | Code | Verified by |
|---|---|---|
| `TMUXS_SOCKET` | `src/tmux.rs` `SOCKET_ENV`, `Tmux::command` (`-L`), `Tmux::new` (empty is unset) | code; unit test `command_honors_the_socket`; all real-tmux tests |
| `TMUXS_CC` (`1`/`0`) | `src/settings.rs` `CC_ENV`, `resolve_iterm2_mode`; read in `src/main.rs` | code; settings tests |
| `TMUXS_CONFIG_DIR` | `src/settings.rs` `CONFIG_DIR_ENV`, `Settings::new` (empty is unset) | code; toggle test writes to a temp dir |
| `TERM_PROGRAM` = `iTerm.app` | `src/main.rs` and `resolve_iterm2_mode` | code; settings tests |
| `TMUX` non-empty | `src/main.rs`, `src/attach.rs` `is_nested` | code; tests `nested_when_tmux_env_is_set_and_non_empty`, `enter_is_refused_inside_tmux_in_both_modes` |
| `SHELL` (basename) | `src/events.rs` `run`, `src/parse.rs` `shell_basename` | code |
| `HOME` (`~` expansion) | `src/tmux.rs` `create_session`, `expand_tilde` | code; test `expands_a_leading_tilde_only` |

### Main-screen keys (`App::handle_key`, `src/app.rs`)

| Key | Code | Test |
|---|---|---|
| Ctrl-R, F2 rename | `app.rs` `open_rename` | events tests `rename_*` |
| Ctrl-N create | `app.rs` `Popup::create` | `create_*` |
| Ctrl-X, Delete kill | `app.rs` `open_kill` | `kill_*` |
| Enter attach | `app.rs` `request_attach` | `enter_requests_attach_*`, `enter_is_refused_*` |
| Ctrl-T, F3 toggle | `app.rs` `toggle_iterm2_mode` | `ctrl_t_and_f3_toggle_and_save_*` |
| Esc, Ctrl-C quit | `app.rs` | `esc_and_ctrl_c_quit_*` |
| Up/Down, Ctrl-K/Ctrl-J | `app.rs` `move_selection` | `arrows_and_ctrl_jk_move_and_clamp` |
| Backspace edits the Search query | `app.rs` | `typing_narrows_the_table_and_backspace_widens_it` |
| Printable characters | `app.rs` (no Ctrl, no Alt) | `control_chords_do_not_edit_the_query` |

### Popup keys (`Popup::cancels`, `Popup::handle_key`, `edit`; `src/popup.rs`)

| Key | Code | Test |
|---|---|---|
| Esc, Ctrl-C close the Popup first | `cancels`; `App::handle_popup_key` | `popups_capture_typing_and_esc_quits_only_without_one` |
| Kill: `y`/`Y` confirms | `handle_key` Kill arm | `kill_needs_y_and_spares_the_prefix_neighbour` |
| Kill: Enter, `n`/`N` cancel | `cancels` | popup tests (Enter, `n`, `x` do not kill) |
| Rename/Create: Enter submits, empty name rejected | `handle_key` | popup tests |
| Create: Tab, Shift-Tab, Up, Down switch field | `handle_key` Create arm | popup tests |
| Text fields: character append, Backspace | `edit` | popup tests |

### Behaviors

| Item | Code |
|---|---|
| Sorting by name then id | `src/parse.rs` `sort_sessions` |
| 1 second refresh | `src/events.rs` `REFRESH_INTERVAL`, `event::poll` timeout |
| Pane state | `src/parse.rs` `pane_state`, `SHELLS` |
| Activity age | `src/parse.rs` `format_age` |
| Directory column dropped | `src/ui.rs` `show_path` |
| Substring filter | `src/parse.rs` `matches_query` |
| `$id` targets | `src/tmux.rs` `rename_session`, `kill_session`, `src/attach.rs` |
| Terminal restored before tmux | `src/events.rs` `attach` |
| Settings file `iterm2_mode` | `src/settings.rs` `File` |

### Mismatches found and fixed in the docs

- Selection is tracked by Session id, not by name (the original spec said name). Docs say id. Behavior is the same or better because ids are unique.
- The Directory column is dropped when no visible Session reports a path; the code does not run a separate tmux capability check. Docs describe the real rule.
- A missing `tmux` does not make the program exit: the error is shown inside the picker and the refresh keeps retrying. Docs describe this. Whether to exit instead is a possible future change, not a bug.
- The Status bar hints list only part of the bindings (no F2, F3, Delete, Ctrl-J/K). Docs say so.
- No code bugs were found during the cross-check.

## Manual verification for the owner

These cannot be automated and were not tried by hand:

1. Attach with iTerm2 mode OFF in a real terminal: plain tmux opens, detaching (prefix + d) returns to the picker with a refreshed list.
2. Attach with iTerm2 mode ON in iTerm2: `tmux -CC attach` opens native tabs; Shell > tmux > Detach returns to the picker.
3. The toggle key in a real iTerm2: Ctrl-T, and whether F3 (and F2) reach the program or are intercepted. The Status bar must flip immediately and the choice must survive a restart.
4. How the UI looks in iTerm2: colors, the selected-row highlight, the cursor in the search bar, box-drawing characters, and the Popups.
5. The control window is hidden when "Automatically bury the tmux client session after connecting" is enabled, and how you return to the picker.
6. Popups on small terminals (Create and Kill with the iTerm2 warning and an error line), and a narrow terminal for the Status bar.
7. A real `-CC` attach by id (`tmux -CC attach -t '$N'`): automatically only the argument parsing is proven.
8. The panic hook: add `panic!()` right after `terminal::setup()`, run in a terminal, confirm the shell stays usable (then remove it).
9. Running `tmuxs` inside tmux: Enter shows the detach-first message in both modes.
