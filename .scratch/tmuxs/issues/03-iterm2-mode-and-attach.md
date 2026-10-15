# 03: iTerm2 mode and Attach

**What to build:** Settings and Attach. The iTerm2 mode is resolved at startup in order: `--cc`/`--no-cc`, then `TMUXS_CC` (`1`/`0`), then the value saved in `settings.toml` under `~/.config/tmuxs/` (resolved with etcetera; `TMUXS_CONFIG_DIR` overrides the directory), then an automatic default (ON when `TERM_PROGRAM` equals `iTerm.app`, otherwise OFF). A missing, unreadable or corrupt file falls back to automatic without a crash. Ctrl-T or F3 toggles the mode, saves it, and the Status bar shows `iTerm2 mode: ON` or `OFF` immediately. Enter attaches: one pure function takes the mode and the Session id and returns the program and arguments (`tmux [-L socket] [-CC] attach -t <id>`). The program restores the terminal, spawns tmux and waits, then re-enters the TUI and refreshes. If `$TMUX` is set, Attach is refused with a message asking the user to detach first, in both modes. `--help` documents flags and environment variables. Verify during implementation that attach with an id target works in both forms.

**Blocked by:** 01

**Status:** ready-for-agent

- [x] Tests for the priority order of flag, `TMUXS_CC`, saved value and automatic default, plus missing and corrupt files, with `TMUXS_CONFIG_DIR` pointing at a temp dir
- [x] Attach-command tests for both modes, with and without a socket
- [x] TestBackend tests show the Status bar for both modes and the `$TMUX` refusal message
- [x] Toggling persists to the settings file in the temp dir, never the real one
- [x] Quality gates pass and the work is committed on `main`
- [x] Manual checklist items for the owner (real attach ON and OFF, toggle key in iTerm2) are listed in the ticket's closing note

## Closing note

Done (committed on a worktree branch, to be merged into `main`). Manual checks for the owner (cannot be automated):

- Enter on a Session with iTerm2 mode OFF: plain tmux attaches in the current terminal; detaching returns to the picker with the list refreshed.
- Enter with iTerm2 mode ON in iTerm2: `tmux -CC attach` opens native tabs; detaching (Shell > tmux > Detach or the control menu) returns to the picker.
- Ctrl-T and F3 toggle the mode inside a real iTerm2 (F3 may be intercepted by macOS or iTerm2 key mappings); the Status bar flips immediately and the choice survives a restart.
- Running `tmuxs` inside tmux: Enter shows the detach-first message in both modes.
- Verified automatically: tmux 3.8 accepts `attach -t '$N'` (plain form resolves the id; an unknown id says "can't find"). With `-CC` and no terminal, tmux fails with "tcgetattr failed" before resolving the target, so the `-CC` id target is only proven to parse.
