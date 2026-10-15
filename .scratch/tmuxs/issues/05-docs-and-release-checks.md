# 05: Documentation and release checks

**What to build:** A short root `README.md` linking to a `docs/` folder: index (what it is, install, quick start), usage (workflows, flags, environment variables `TMUXS_SOCKET`, `TMUXS_CC`, `TMUXS_CONFIG_DIR`), key bindings (matching the code exactly), iTerm2 mode (what `-CC` is, what the toggle does, hiding the control window via "Automatically bury the tmux client session after connecting", the pitfall that closing an iTerm2 tab, window or session kills the matching tmux window or Session while Shell > tmux > Detach keeps it running, and a list of names that could not be verified), architecture (modules, data flow, why the terminal is restored before spawning tmux), development (build, test, lint, safe testing with the two test variables) and troubleshooting (no server, running inside tmux, garbled terminal after a crash, settings problems). Keep the existing `docs/agents/` folder. Document only verified behavior and mark the rest unverified. Cross-check keys, flags and variables against the implementation. Finish with `cargo build --release`, install instructions (`cargo install --path .`, no shell profile edits), and the final short report: project path, test results, what was imitated from `sshs`, deviations from the original prompt, what was tested automatically, and what the owner must try by hand (attach ON and OFF, toggle key, appearance in iTerm2, control window hidden with the bury setting).

**Blocked by:** 02, 03, 04

**Status:** done

- [x] All docs are in English and every listed page exists
- [x] A documented cross-check shows keys, flags and environment variables match the code
- [x] Unverified items are explicitly marked
- [x] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` and `cargo build --release` pass
- [x] Final report written, with the manual verification checklist
- [x] Committed on `main`

## Closing note

Done. Added `README.md` and `docs/{index,usage,keybindings,iterm2-cc-mode,architecture,development,troubleshooting}.md` (`docs/agents/` untouched). The cross-check checklist is in `docs/development.md`; it found no code bugs, only doc/spec wording differences (selection by id, Directory column rule, missing tmux shown in the picker instead of exiting). Gates passed: fmt, clippy `--all-targets -D warnings`, 81 tests, release build. Install with `cargo install --path .` (not run).

Manual checks for the owner (also in `docs/development.md`):

- Attach with iTerm2 mode ON and OFF; detaching returns to the picker.
- Toggle key in real iTerm2, including whether F3 (and F2) are intercepted.
- How the UI looks in iTerm2.
- Control window hidden when "Automatically bury the tmux client session after connecting" is enabled.
- Popups on small terminals and the Status bar on narrow terminals.
- A real `-CC` attach by id.
- The panic hook, and the nested-tmux refusal in both modes.
