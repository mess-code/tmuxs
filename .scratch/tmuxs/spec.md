# tmuxs: a tmux Session picker (v1)

Status: ready-for-agent

Vocabulary follows `CONTEXT.md` (Session, Window, Pane, Pane state, Attach, iTerm2 mode, Search query, Activity age, Popup, Status bar). Background facts and citations are in `.scratch/tmuxs/research.md`. Everything in the project, including code, comments, tests, docs, `--help`, commit messages and reports, is written in English.

## Problem Statement

I keep my long-running agent work in tmux Sessions and I use only a terminal and iTerm2, with no other GUI tools. Finding the right Session, seeing which ones still have something running, and attaching to, creating, renaming or killing them means remembering names and typing several tmux commands. `sshs` makes this painless for SSH hosts, but nothing equivalent exists for tmux Sessions, and nothing helps me choose between iTerm2's tmux control mode (`-CC`) and a plain tmux client when I attach.

## Solution

A small Rust terminal program, `tmuxs` (package and binary), that looks and feels like `sshs`. It shows an always-focused search bar, a table of every tmux Session, a one-line summary of the selected Session's Windows, and a Status bar. From it I can Attach to, create, rename and kill Sessions with single key chords, and I can switch iTerm2 mode on and off at any time. The table shows, for each Session, how many Windows and Panes it has and how many Panes are busy, idle or dead, so I can see at a glance which agents are still working. After I detach from tmux, I land back in the picker.

## User Stories

1. As a tmux user, I want to run `tmuxs` in a normal terminal and see every tmux Session in a table, so that I do not have to remember Session names.
2. As a user with no running tmux server, I want an empty state that tells me to press Ctrl-N to create a Session, so that the program never looks broken or crashes.
3. As a user with Sessions, I want the table to have a header row and a highlighted selected row, so that I always know which Session an action will apply to.
4. As a user, I want the table sorted by name, so that rows do not jump around as Sessions produce output.
5. As a user, I want each row to show the Session name, so that I can identify it.
6. As a user, I want each row to show the number of Windows, so that I know how many iTerm2 tabs an iTerm2 mode Attach will open.
7. As a user, I want each row to show the number of Panes, so that I know how many terminals the Session contains.
8. As a user running agents, I want each row to show how many Panes are busy, idle and dead (for example `2 busy · 1 idle`), so that I can see which Sessions still have work running.
9. As a user, I want a Pane to count as idle when its foreground command is a shell (`sh`, `bash`, `zsh`, `fish`, `dash`, `ksh`, `tcsh`, `nu`, or the basename of `$SHELL`), so that "sitting at a prompt" is distinguishable from "running something".
10. As a user, I want a Pane whose process has exited to count as dead, so that finished or crashed work is visible.
11. As a user, I want each row to show whether the Session is attached or detached, in different colors, so that I can see where I am already connected.
12. As a user, I want each row to show the Activity age as a short relative duration (`5s`, `12m`, `3h`, `2d`), so that I can tell which Sessions are recently active.
13. As a user, I want each row to show the Session's working directory when the installed tmux supports it, so that I can tell Sessions with similar names apart; if the installed tmux lacks the format variable, I want that column dropped rather than shown broken.
14. As a user, I want a one-line summary of the selected Session's Windows (for example `0:zsh 1:claude 2:vim`) above the Status bar, so that I can see what is inside it without attaching.
15. As a user, I want a search bar at the top that is always focused, so that I can start typing immediately without a mode switch.
16. As a user, I want my Search query to narrow the table by case-insensitive substring match on the Session name as I type, so that I can find a Session in a few keystrokes.
17. As a user, I want Backspace to edit my Search query, so that I can correct typing mistakes.
18. As a user, I want the selection to stay on the same Session (by name) when the Search query or the list changes, falling back to a sensible row if that Session disappears, so that I never act on the wrong Session.
19. As a user with Sessions named `cat` and `category`, I want actions on `cat` never to affect `category`, so that I cannot kill or attach the wrong one.
20. As a user, I want Sessions whose names contain spaces, dots, colons or other unusual characters to be listed and fully operable, so that odd names never break the tool.
21. As a user, I want to move the selection with the Up and Down arrows and with Ctrl-J and Ctrl-K, so that I can navigate in the style of `sshs`.
22. As a user, I want Enter to Attach to the selected Session, so that I can start working in it.
23. As a user in iTerm2 with iTerm2 mode ON, I want Enter to run tmux in control mode (`-CC`), so that iTerm2 shows the Session as native tabs and splits.
24. As a user with iTerm2 mode OFF, I want Enter to run a plain tmux Attach, so that I can use tmux inside the current terminal.
25. As a user, I want the terminal fully restored (raw mode off, alternate screen left, cursor visible) before tmux starts, so that tmux gets a clean terminal.
26. As a user, I want `tmuxs` to wait for the tmux client to finish and then return to the picker with a refreshed list, so that I can hop between Sessions after detaching.
27. As a user already inside tmux (`$TMUX` is set), I want Attach refused with a message asking me to detach first, in both modes, so that I do not create nested tmux clients.
28. As a user, I want a panic anywhere in the program to still restore my terminal, so that a crash never leaves my shell garbled.
29. As a user, I want Ctrl-T or F3 to toggle iTerm2 mode at runtime, so that I can pick the Attach style per Session.
30. As a user, I want the Status bar to show `iTerm2 mode: ON` or `iTerm2 mode: OFF` immediately after toggling, so that I always know how Enter will behave.
31. As a user, I want the iTerm2 mode to start from the command-line flag `--cc` or `--no-cc` first, then the `TMUXS_CC` environment variable (`1`/`0`), then my last saved choice, then an automatic default (ON when `TERM_PROGRAM` is `iTerm.app`, otherwise OFF), so that it does the right thing with no configuration and is still overridable.
32. As a user, I want my last toggled iTerm2 mode saved to `~/.config/tmuxs/settings.toml`, so that it persists between runs.
33. As a user, I want a missing or corrupt settings file to fall back to the automatic default without a crash, so that a bad file never blocks me.
34. As a tester, I want `TMUXS_CONFIG_DIR` to override the settings directory, so that tests never touch my real settings.
35. As a tester, I want `TMUXS_SOCKET` to make every tmux command run against `tmux -L <value>`, so that tests never touch my real Sessions.
36. As a user, I want Ctrl-N to open a Popup to create a Session with a name and a directory, so that I can start a Session for a new feature.
37. As a user, I want the create Popup's name to be prefilled from my current Search query, so that searching for a nonexistent name and pressing Ctrl-N creates it.
38. As a user, I want the create Popup's directory to default to the directory I launched `tmuxs` from, with `~` expanded, so that new Sessions start where I am.
39. As a user, I want an empty name rejected inside the create Popup, so that I never create a nameless Session.
40. As a user, I want a duplicate name or a bad directory to keep the create Popup open and show tmux's error inside it, so that I can fix the input without retyping.
41. As a user, I want Ctrl-R or F2 to open a rename Popup prefilled with the current name, so that I can rename quickly.
42. As a user, I want rename errors (duplicate name, missing Session) shown inside the Popup, which stays open, so that I can correct and retry.
43. As a user, I want Ctrl-X or Delete to open a kill confirmation Popup (y/N, default No), so that I cannot kill a Session by accident.
44. As a user with iTerm2 mode ON, I want the kill confirmation to warn that the Session's iTerm2 tabs will close, so that I understand the consequence.
45. As a user, I want a failed kill (for example the Session vanished) to show the tmux error in the Popup or Status bar, so that I know what happened.
46. As a user, I want Esc or Ctrl-C to close an open Popup first and quit the program only when no Popup is open, so that I can back out safely.
47. As a user, I want the list refreshed automatically every second and after every action, so that the table reflects reality without a manual refresh.
48. As a user, I want the Status bar to show key hints, the current iTerm2 mode, and result messages such as "Renamed" or tmux errors, so that I get feedback for every action.
49. As a user without `tmux` on my PATH, I want a clear, understandable error instead of a crash, so that I know what to install.
50. As a user, I want `--help` to describe the flags and environment variables, so that I can discover them.
51. As a user, I want a README and a `docs/` folder that describe usage, key bindings, iTerm2 mode, architecture, development and troubleshooting, so that I can learn and maintain the tool.
52. As a user, I want the docs to mark every behavior that was not verified, so that I know what to check by hand.
53. As the owner, I want a clear checklist of what I must verify manually in iTerm2, so that I know what automated tests could not cover.
54. As the owner, I want to install the binary with `cargo install --path .` without the build touching my shell profile, so that my environment stays under my control.

## Implementation Decisions

**Modules** (small and readable; exact file layout is up to the implementer): a tmux module that runs tmux through `std::process::Command` and honors `TMUXS_SOCKET`; a parsing module with pure functions; an app/state module holding the model and key handling; a settings module; a rendering module; an event loop / terminal-handling module; and the entry point with the CLI definition.

**Crates**: ratatui and crossterm (use ratatui's re-export), serde plus toml for settings, etcetera for the config directory, clap for flags. Add every dependency with `cargo add` to get the current version; do not copy versions from memory or from the research file. Read the docs of the versions actually installed. Filtering is plain case-insensitive substring, written as one pure function so a fuzzy matcher could replace it later.

**tmux facts verified on tmux 3.8** (see research file):
- Session data comes from `list-sessions -F` with a tab delimiter. tmux rejects tabs in Session names, so a tab is a safe separator. Fields needed: session id, name, window count, attached count, activity (epoch seconds), and the working directory variable if supported.
- Pane data comes from `list-panes -a -F` with session id, window index, window name, whether the pane is dead, and the foreground command. Per-pane activity variables are empty in 3.8; do not rely on them.
- `-t =name` does not work for pane-taking commands (it needs `=name:`), and names containing `.` or `:` cannot be targeted by name at all. Therefore every command targets a Session by its id (the `$N` form from the session id variable). This is exact by construction, so `cat` never matches `category`. This deliberately replaces the `=<name>` wording in the original prompt.
- An empty name is accepted by tmux, so `tmuxs` validates non-empty names itself in the create and rename Popups.
- Attach, new-session and similar commands run from inside a tmux client fail with "sessions should be nested with care"; `tmuxs` checks `$TMUX` first and shows its own message.
- Verify during implementation that attach works with an id target in both plain and `-CC` forms.
- Check which format variables the installed tmux supports at startup; drop the directory column if the working-directory variable is unsupported.

**Pane state derivation** is a pure function: dead if the dead flag is set; idle if the foreground command is in the fixed shell list plus the basename of `$SHELL`; busy otherwise. This is a heuristic (an agent waiting for input still counts as busy) and must be documented as such.

**Activity age** is computed from the Session's activity epoch and the current time, formatted in seconds, minutes, hours or days, with no date library.

**Attach behavior**: one pure function takes the iTerm2 mode and the Session id and returns the program and its arguments (`tmux [-L socket] [-CC] attach -t <id>`). The program: restores the terminal (raw mode off, alternate screen left, cursor shown), spawns the command, waits for it to exit, re-enters the TUI and refreshes. This follows `sshs`, which spawns and waits instead of replacing the process; it deliberately replaces the `exec` wording in the original prompt so that detaching returns to the picker. A panic hook always restores the terminal. If `$TMUX` is set, Attach is refused with a message in both modes.

**Input model**: the search bar is always focused and every printable key edits the Search query. All actions use Ctrl or function keys. Key bindings: Enter attach; Ctrl-R or F2 rename; Ctrl-X or Delete kill (with confirmation); Ctrl-N create; Ctrl-T or F3 toggle iTerm2 mode; Up/Down and Ctrl-J/Ctrl-K move; Backspace edits the query; Esc or Ctrl-C closes a Popup first, then quits. Where `sshs` uses additional navigation keys that do not conflict, adopt them (see the research file's key-binding section) and document them.

**Layout**: three regions in the style of `sshs`: search bar, table, and a bottom area with the selected Session's Window summary line above the Status bar. Popups are centered overlays. Table columns: Name, Windows, Panes (busy/idle/dead), State, Activity, Directory.

**Popups**: rename (single text field), create (name and directory fields), kill confirmation (y/N, default No, extra warning when iTerm2 mode is ON). Rename and create errors stay inside the Popup and keep it open. Successes and non-dialog errors go to the Status bar.

**Refresh and selection**: the list is re-read every second and after every action. The selection is tracked by Session name so it survives refresh, filtering and reordering.

**Settings**: the only persisted setting is the iTerm2 mode, stored in `settings.toml` under `~/.config/tmuxs/` (resolved with etcetera so it is XDG-style on macOS), or under `TMUXS_CONFIG_DIR` when set. Initial mode priority: `--cc`/`--no-cc`, then `TMUXS_CC` (`1`/`0`), then the saved value, then automatic (`TERM_PROGRAM` equals `iTerm.app` gives ON). A missing, unreadable or corrupt file falls back to automatic. Toggling saves the new value.

**Environment variables**: `TMUXS_SOCKET` (run tmux with `-L <value>`), `TMUXS_CC` (`1`/`0`), `TMUXS_CONFIG_DIR`.

**Errors**: no tmux server or no Sessions gives an empty state with a Ctrl-N hint. `tmux` missing from PATH gives a plain-language error and a non-crashing exit.

**Documentation**: a short root `README.md` linking to a `docs/` folder with: index (what it is, install, quick start), usage (workflows, flags, environment variables), key bindings (matching the code exactly), iTerm2 mode (what `-CC` is, what the toggle does, hiding the control window with "Automatically bury the tmux client session after connecting", the pitfall that closing an iTerm2 tab, window or session kills the matching tmux window or Session while Shell > tmux > Detach keeps it running, and a list of names that could not be verified), architecture (modules, data flow, why the terminal is restored before spawning tmux), development (build, test, lint, safe testing with the two test environment variables), and troubleshooting (no server, running inside tmux, garbled terminal after a crash, settings file problems). The existing `docs/agents/` folder is separate and must not be removed. Only document verified behavior; mark everything else unverified. Before finishing, cross-check keys, flags and variables in the docs against the implementation.

**Installation**: tell the user to run `cargo install --path .`; do not edit shell profiles.

## Testing Decisions

A good test exercises external behavior only: keys in, rendered buffer and tmux state out. It does not assert on private structure.

**Main seam (the whole app)**: build the app against a real tmux server on the throwaway socket `tmuxs-test` with `TMUXS_CONFIG_DIR` set to a temp directory, send key events, and assert on the `TestBackend` buffer and on actual tmux state. Clean up with `tmux -L tmuxs-test kill-server`. This covers: list, search narrowing, the `cat`/`category` case, empty state, rename, kill with confirmation (including the extra iTerm2 warning), create (including duplicate-name and missing-Session error paths and inline Popup errors), selection kept by name across refresh, and the Status bar showing iTerm2 mode in both states.

**Pure seam 1, parsing and derivation**: `list-sessions` and `list-panes` parsing (names with spaces, dots, colons and other unusual characters; many Sessions), Pane state derivation (busy, idle, dead, `$SHELL` basename), Activity age formatting, and the substring filter.

**Pure seam 2, settings and attach command**: iTerm2 mode resolution across flag, `TMUXS_CC`, saved value and automatic default; settings load and save including missing and corrupt files; the attach command for both modes, including the socket option and the `$id` target.

**Quality gates** before reporting done: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release`. Tests must never touch the real tmux server or the real settings.

**Prior art**: none in this repo yet (it is empty). `sshs` is the behavioral reference; ratatui's `TestBackend` is the rendering test tool.

**Cannot be tested automatically** (manual checklist for the owner): Attach in iTerm2 mode ON and OFF, the toggle key inside a real iTerm2, how the UI looks in iTerm2, and the control window being hidden when the bury setting is enabled.

## Out of Scope

- Rewriting a terminal, or managing Panes and Windows beyond displaying them.
- A preview of Session screen contents (`capture-pane`); this was dropped in favor of Pane state counts.
- Any agent-status column beyond the busy/idle/dead heuristic.
- Fuzzy matching, other sort orders, and any persisted setting other than iTerm2 mode.
- Windows, Linux or other terminals as supported targets.
- Publishing the crate or a remote repository.

## Further Notes

- Deviations from the original prompt, all agreed: spawn-and-return instead of `exec`; Session id targets instead of `=<name>`; no preview pane; Pane state counts instead.
- Unverified items from research: the exact nested-refusal message in `-CC` mode, whether `assert_buffer_eq!` still exists in current ratatui (use the documented buffer assertions instead), and the source-level identifier of the iTerm2 bury setting.
- `cargo`, `clippy` and `rustfmt` are installed via Homebrew (Rust 1.99).
- Git: one commit per ticket on `main`, English messages ending with the Co-Authored-By line given by the harness.
