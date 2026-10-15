# 03: iTerm2 mode and Attach

**What to build:** Settings and Attach. The iTerm2 mode is resolved at startup in order: `--cc`/`--no-cc`, then `TMUXS_CC` (`1`/`0`), then the value saved in `settings.toml` under `~/.config/tmuxs/` (resolved with etcetera; `TMUXS_CONFIG_DIR` overrides the directory), then an automatic default (ON when `TERM_PROGRAM` equals `iTerm.app`, otherwise OFF). A missing, unreadable or corrupt file falls back to automatic without a crash. Ctrl-T or F3 toggles the mode, saves it, and the Status bar shows `iTerm2 mode: ON` or `OFF` immediately. Enter attaches: one pure function takes the mode and the Session id and returns the program and arguments (`tmux [-L socket] [-CC] attach -t <id>`). The program restores the terminal, spawns tmux and waits, then re-enters the TUI and refreshes. If `$TMUX` is set, Attach is refused with a message asking the user to detach first, in both modes. `--help` documents flags and environment variables. Verify during implementation that attach with an id target works in both forms.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Tests for the priority order of flag, `TMUXS_CC`, saved value and automatic default, plus missing and corrupt files, with `TMUXS_CONFIG_DIR` pointing at a temp dir
- [ ] Attach-command tests for both modes, with and without a socket
- [ ] TestBackend tests show the Status bar for both modes and the `$TMUX` refusal message
- [ ] Toggling persists to the settings file in the temp dir, never the real one
- [ ] Quality gates pass and the work is committed on `main`
- [ ] Manual checklist items for the owner (real attach ON and OFF, toggle key in iTerm2) are listed in the ticket's closing note
