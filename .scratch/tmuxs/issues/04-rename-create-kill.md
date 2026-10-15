# 04: Rename, create and kill

**What to build:** Popups for managing Sessions. Ctrl-R or F2 opens a rename Popup prefilled with the current name. Ctrl-N opens a create Popup with name and directory fields; the name is prefilled from the current Search query, and the directory defaults to the launch directory with `~` expanded. Empty names are rejected inside the Popup. Duplicate-name, bad-directory and missing-Session errors from tmux are shown inside the Popup, which stays open. Ctrl-X or Delete opens a kill confirmation (y/N, default No) with an extra warning that the Session's iTerm2 tabs will close when iTerm2 mode is ON. Esc or Ctrl-C closes an open Popup first and quits only when none is open. Successes show messages such as "Renamed" in the Status bar. The list refreshes after every action and the selection follows by name. All commands target `$id`.

**Blocked by:** 02

**Status:** ready-for-agent

- [x] End-to-end tests on `TMUXS_SOCKET=tmuxs-test` (keys in, TestBackend buffer and real tmux state out) for rename, create and kill
- [x] Error paths: duplicate name, nonexistent Session, empty name, bad directory
- [x] TestBackend tests for the confirmation Popup with iTerm2 mode ON and OFF
- [x] `cat` and `category` cases: renaming or killing one never affects the other
- [x] Quality gates pass and the work is committed on `main`

## Closing note

Implemented in `src/popup.rs` (Popup state, `Action`), `App::handle_key` (state only, sets `pending_action`), `events::run_pending` (runs tmux, refreshes, reports) and `Tmux::{create_session, rename_session, kill_session}`. tmux 3.8 accepts an empty name and a nonexistent directory, so both are rejected by `tmuxs` (empty name inside the Popup, directory by an `is_dir` check before `new-session`). Create fields switch with Tab, Shift-Tab, Up or Down. In the kill confirmation Enter, `n`, Esc and Ctrl-C all cancel; only `y` kills. Hints use `^N`/`^R`/`^X` to fit the Status bar.
