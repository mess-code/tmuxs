# 02: Search, Pane state and live data

**What to build:** The always-focused search bar narrows the table by case-insensitive substring on the Session name as the user types (Backspace edits the Search query), as one pure function. The table gains Panes (busy/idle/dead counts), Activity age (`5s`, `12m`, `3h`, `2d`, from the activity epoch, no date crate) and Directory columns; Directory is dropped if the installed tmux does not support the variable. Pane data comes from `list-panes -a -F` (Session id, Window index and name, dead flag, foreground command). Pane state is a pure function: dead, else idle if the foreground command is in the fixed shell list (`sh`, `bash`, `zsh`, `fish`, `dash`, `ksh`, `tcsh`, `nu`) or equals the basename of `$SHELL`, else busy. A one-line summary of the selected Session's Windows (for example `0:zsh 1:claude`) sits above the Status bar, which shows key hints. The list refreshes every second, and the selection follows the same Session by name across refresh and filtering. Every tmux command targets Sessions by `$id`.

**Blocked by:** 01

**Status:** ready-for-agent

- [x] Pure tests: filter, Pane state (busy, idle, dead, `$SHELL` basename), Activity age formatting
- [x] Parsing tests for `list-panes` output with unusual window names
- [x] TestBackend tests for the filtering state and the Status bar hints
- [x] Real-tmux test: typing `cat` shows `cat` and not `category` unless it matches as a substring (substring semantics documented), selection stays on the same Session by name after a refresh
- [x] Real-tmux test: a Session with a busy pane and an idle pane shows the right counts
- [ ] Quality gates pass and the work is committed on `main`
