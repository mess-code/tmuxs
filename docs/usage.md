# Usage

## The screen

From top to bottom:

1. Search bar (3 rows, always focused). Typing edits the Search query.
2. Table of Sessions, or an empty-state message, or a red tmux error.
3. Status bar: key hints (or the latest result message) followed by `| iTerm2 mode: ON` or `| iTerm2 mode: OFF`.

### Table columns

| Column | Meaning |
|---|---|
| Name | Session name, verbatim (spaces, dots, colons and so on are fine). |
| Windows | Number of Windows. |
| Panes | Total Panes, then the non-zero counts per Pane state, for example `3 (2 busy · 1 idle)`. If no Pane data is available only the total is shown. |
| State | `attached` (green) if at least one client is attached, otherwise `detached` (yellow). |
| Activity | Activity age, see below. |
| Directory | The Session's working directory (`session_path`). |

Sorting: by name (plain string comparison, so uppercase sorts before lowercase), then by Session id. Rows do not reorder as Sessions produce output.

Directory column dropping rule: the column is shown only if at least one visible Session reports a non-empty `session_path`. tmux expands an unknown format variable to an empty string, so a tmux without the variable simply yields an empty column, and `tmuxs` then drops it. The code does not run a separate version check.

### Pane state

Each Pane is classified by `pane_state`:

- dead: tmux reports the Pane's process has exited (`pane_dead` is 1).
- idle: the foreground command (`pane_current_command`) is one of `sh`, `bash`, `zsh`, `fish`, `dash`, `ksh`, `tcsh`, `nu`, or equals the basename of `$SHELL`. A leading `-` (login shell, `-zsh`) is ignored.
- busy: anything else.

This is a heuristic. A busy Pane may be an agent that is just waiting for input; an idle Pane may be a shell with a background job. tmux 3.8 per-Pane activity variables are empty, so they are not used.

### Activity age

Computed from the Session's `session_activity` epoch and the current time: `0`-`59` seconds as `Ns`, then `Nm` (minutes, rounded down), `Nh`, and `Nd` for a day or more. A negative difference (clock skew) shows `0s`.

## Workflows

- Find: type any part of a Session name. Matching is a case-insensitive substring match on the name only. The selection stays on the same Session (tracked by its tmux id, not its position) while you type, unless that Session is filtered out; then it falls back to a nearby row.
- Attach: Enter. See below.
- Create: Ctrl-N opens a Popup with Name (prefilled from the Search query) and Directory (prefilled with the directory you launched `tmuxs` from). A leading `~` or `~/` in the directory is expanded using `$HOME`. An empty name is rejected inside the Popup. A directory that does not exist is rejected by `tmuxs` itself (tmux 3.8 would accept it). A duplicate name shows tmux's error inside the Popup, which stays open. On success the Session is created detached, selected (the Search query is cleared if it would hide it) and the Status bar says "Created". Creating works even while `tmuxs` runs inside tmux because it only creates a detached Session.
- Rename: Ctrl-R or F2 opens a Popup prefilled with the current name. Empty names are rejected; tmux errors stay inside the Popup. Status bar: "Renamed".
- Kill: Ctrl-X or Delete opens a confirmation. Only `y` or `Y` kills. With iTerm2 mode ON the Popup adds a warning that the Session's iTerm2 tabs will close. Status bar: "Killed". A failed kill (for example the Session vanished) shows tmux's error inside the Popup.
- Toggle iTerm2 mode: Ctrl-T or F3. The Status bar flips at once and the choice is saved.
- Quit: Esc or Ctrl-C when no Popup is open.

If saving the setting fails, the Status bar shows "Could not save settings: ..." and the mode stays toggled for this run.

All actions target a Session by its tmux id (`$N`), never by name. Reason (tmux 3.8, from the research notes and tests): `-t =name` does not work for Pane-taking commands, names containing `.` or `:` cannot be targeted by name at all, and a bare name does prefix matching, so `cat` could hit `category`. Ids are exact. tmux 3.8 also accepts an empty Session name, which is why `tmuxs` validates non-empty names itself.

## Attach

Enter runs `tmux [-L <socket>] [-CC] attach -t <id>`:

1. The Terminal is restored first: raw mode off, alternate screen left, cursor shown. tmux needs a clean terminal to draw its own UI.
2. `tmuxs` spawns tmux and waits for it to exit (this follows `sshs`; it does not `exec`).
3. It re-enters raw mode and the alternate screen, refreshes the list and shows the picker again. If tmux exited with a non-zero status the Status bar says `tmux exited with <status>`; if tmux could not be started it says `Could not run tmux: <error>`.

If `$TMUX` is set and non-empty (you are inside tmux), Attach is refused in both modes with: "Already inside tmux. Detach first (prefix + d), then run tmuxs." The refusal is decided by `tmuxs` before it runs tmux. Create, rename and kill are not refused.

## Command-line flags

Exactly as `tmuxs --help` prints them:

```
A terminal picker for tmux Sessions.

Environment variables:

TMUXS_SOCKET     run every tmux command as `tmux -L <value>`.

TMUXS_CC         iTerm2 mode at startup: 1 is ON, 0 is OFF. Overridden by --cc and --no-cc, and overrides the saved choice.

TMUXS_CONFIG_DIR directory holding settings.toml (default ~/.config/tmuxs).

Without any of these, iTerm2 mode is the saved choice, or ON when TERM_PROGRAM is iTerm.app and OFF otherwise. Ctrl-T or F3 toggles it.

Usage: tmuxs [OPTIONS]

Options:
      --cc
          Attach with iTerm2 mode ON (tmux control mode, `tmux -CC`)

      --no-cc
          Attach with iTerm2 mode OFF (a plain tmux client)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

`--cc` and `--no-cc` conflict; giving both is a usage error. The flags do not write the saved setting by themselves; only toggling with Ctrl-T or F3 saves.

## Environment variables

| Variable | Effect |
|---|---|
| `TMUXS_SOCKET` | Every tmux command (list, create, rename, kill, attach) runs as `tmux -L <value>`. Empty counts as unset. Meant for tests. |
| `TMUXS_CC` | `1` is iTerm2 mode ON, `0` is OFF. Any other value is ignored. |
| `TMUXS_CONFIG_DIR` | Directory that holds `settings.toml`. Empty counts as unset. Meant for tests. |

`TERM_PROGRAM` is also read (not set by `tmuxs`), and `SHELL` (for Pane state), `HOME` (for `~` expansion) and `TMUX` (for the nested check).

## iTerm2 mode: initial value

First match wins:

1. `--cc` or `--no-cc`
2. `TMUXS_CC` (`1` or `0`)
3. the saved choice in `settings.toml`
4. automatic: ON when `TERM_PROGRAM` equals `iTerm.app`, otherwise OFF

Caveat (Unverified): inside a tmux Pane `TERM_PROGRAM` may not be `iTerm.app`. This rarely matters because Attach is refused inside tmux anyway.

## Settings file

- Location: `settings.toml` inside `$TMUXS_CONFIG_DIR` if set, otherwise inside the `tmuxs` application config directory chosen by the `etcetera` crate. Its documentation says it uses the XDG strategy on Linux and macOS, which is why the default is `~/.config/tmuxs/settings.toml`; `$XDG_CONFIG_HOME` is honored by the same crate. The default path itself was not probed so as not to touch real settings.
- Format: TOML with one optional key.

```toml
iterm2_mode = true
```

- A missing, unreadable or corrupt file, or a wrong value type, is treated as "no saved choice" and the automatic default applies. The file is created (with its directory) the first time you toggle.

## Refresh

The Session list is re-read when 1 second passes without a key press, and immediately after every action (create, rename, kill) and after returning from Attach. While keys keep arriving faster than once a second, the periodic refresh waits until input pauses. The screen is redrawn every loop iteration.
