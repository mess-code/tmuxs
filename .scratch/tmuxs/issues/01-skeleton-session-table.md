# 01: Skeleton: a Session table you can quit from

**What to build:** A new `tmuxs` crate (package and binary) that shows every tmux Session in a table and exits cleanly. The tmux wrapper runs tmux through `std::process::Command` and honors `TMUXS_SOCKET` (`tmux -L <value>`). It parses `list-sessions -F` (tab delimiter, Session id, name, Window count, attached count) into a struct. The table has a header row, a highlighted selected row, columns Name, Windows and State (attached/detached, colored), sorted by name. Up/Down and Ctrl-J/Ctrl-K move the selection. With no tmux server or no Sessions, an empty state tells the user to press Ctrl-N (the key does nothing yet). If `tmux` is not on PATH, a plain-language error is shown without a crash. Esc or Ctrl-C quits. The terminal is set up and restored properly, and a panic hook restores it. Modules are split (tmux calls, parsing, app state, rendering, event handling). Add all dependencies with `cargo add` and read the docs of the installed versions. All text in English. See `.scratch/tmuxs/spec.md` and `.scratch/tmuxs/research.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` pass
- [ ] Parsing tests cover names with spaces, dots, colons and other unusual characters, and many Sessions
- [ ] TestBackend tests cover the list state and the empty state
- [ ] A real-tmux test on `TMUXS_SOCKET=tmuxs-test` creates Sessions and checks the listing, then cleans up with `tmux -L tmuxs-test kill-server`
- [ ] Sessions `cat` and `category` are both listed and distinguishable by id
- [ ] The panic hook restores the terminal (verified by a test or a documented manual check)
- [ ] Committed on `main` in English with the Co-Authored-By line
