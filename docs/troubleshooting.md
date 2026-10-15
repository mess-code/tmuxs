# Troubleshooting

## "No tmux sessions. Press Ctrl-N to create one."

No tmux server is running, or it has no Sessions. This is normal. Press Ctrl-N to create one. If `TMUXS_SOCKET` is set in your environment, `tmuxs` talks to `tmux -L <value>`, not your default server; unset it to see your usual Sessions.

## "No sessions match the search."

Your Search query hides every Session. Press Backspace to widen it.

## "tmux was not found on your PATH. Install tmux and try again."

Install tmux (for example `brew install tmux` on macOS) and make sure it is on the `PATH` of the shell you start `tmuxs` from. The message is shown in the picker, which keeps checking every second.

## "Already inside tmux. Detach first (prefix + d), then run tmuxs."

`$TMUX` is set, meaning `tmuxs` runs inside a tmux Pane. Attach is refused in both modes to avoid nested tmux clients (tmux itself says "sessions should be nested with care"). Detach, then run `tmuxs` from a plain terminal. Creating, renaming and killing still work inside tmux. If `$TMUX` is set but empty, it is treated as not set.

## The terminal looks garbled after a crash

`tmuxs` installs a panic hook that restores the terminal, but a hard kill (`kill -9`) or a crash of the process cannot run it. Run `reset` (or `stty sane` followed by `tput rmcup; tput cnorm`) to recover. The panic hook has not been exercised by an automated test.

## Enter shows "tmux exited with ..." or "Could not run tmux: ..."

tmux started but returned a non-zero status, or could not be started. The picker returns with a refreshed list. Try the same command by hand: `tmux attach -t '$N'` (with `-L <socket>` if you use `TMUXS_SOCKET`; with `-CC` in iTerm2 mode).

## F2 or F3 does nothing

The terminal or macOS may intercept function keys (Unverified for F3 in iTerm2). Use Ctrl-R for rename and Ctrl-T for the toggle. Ctrl-X also replaces Delete.

## The Directory column is missing

None of the visible Sessions reported a working directory, which is what an old tmux without `session_path` looks like. The column is dropped on purpose.

## iTerm2 mode starts in the wrong state

Check, in order: `--cc` / `--no-cc`, then `TMUXS_CC` (`1`/`0`), then the saved choice, then `TERM_PROGRAM` (`iTerm.app` gives ON). Run `echo $TERM_PROGRAM` and `echo $TMUXS_CC`. See [usage.md](usage.md#iterm2-mode-initial-value).

## Settings problems

- The file is `settings.toml` in `$TMUXS_CONFIG_DIR` or the default `~/.config/tmuxs/`. It contains only `iterm2_mode = true` or `false`.
- A missing, unreadable or corrupt file is ignored (automatic default). Delete it to reset.
- "Could not save settings: ..." means the directory or file is not writable. The toggle still works for the current run but will not persist. Set `TMUXS_CONFIG_DIR` to a writable directory if needed.
- "Could not save settings: no settings directory found" means no home directory could be determined; set `TMUXS_CONFIG_DIR`.

## Closing an iTerm2 tab killed my tmux Window or Session

In iTerm2 mode (`-CC`), closing an iTerm2 tab, window or session kills the matching tmux Window or Session, per iTerm2's documentation. Use Shell > tmux > Detach to leave the Session running. See [iterm2-cc-mode.md](iterm2-cc-mode.md).

## Rename or kill says "can't find session"

The Session vanished between the last refresh and your action. The error is shown in the Popup and the list refreshes; close the Popup and select again.

## Pane counts look wrong

Pane state is a heuristic based on the foreground command; see [usage.md](usage.md#pane-state). An agent waiting for input counts as busy.

## Windows and Linux

macOS is the tested platform. Linux is unverified. Windows is not supported; use WSL with tmux.
