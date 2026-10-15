# 04: Rename, create and kill

**What to build:** Popups for managing Sessions. Ctrl-R or F2 opens a rename Popup prefilled with the current name. Ctrl-N opens a create Popup with name and directory fields; the name is prefilled from the current Search query, and the directory defaults to the launch directory with `~` expanded. Empty names are rejected inside the Popup. Duplicate-name, bad-directory and missing-Session errors from tmux are shown inside the Popup, which stays open. Ctrl-X or Delete opens a kill confirmation (y/N, default No) with an extra warning that the Session's iTerm2 tabs will close when iTerm2 mode is ON. Esc or Ctrl-C closes an open Popup first and quits only when none is open. Successes show messages such as "Renamed" in the Status bar. The list refreshes after every action and the selection follows by name. All commands target `$id`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] End-to-end tests on `TMUXS_SOCKET=tmuxs-test` (keys in, TestBackend buffer and real tmux state out) for rename, create and kill
- [ ] Error paths: duplicate name, nonexistent Session, empty name, bad directory
- [ ] TestBackend tests for the confirmation Popup with iTerm2 mode ON and OFF
- [ ] `cat` and `category` cases: renaming or killing one never affects the other
- [ ] Quality gates pass and the work is committed on `main`
