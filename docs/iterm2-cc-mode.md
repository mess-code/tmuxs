# iTerm2 mode (tmux control mode)

iTerm2 mode is a `tmuxs` setting that decides how Enter Attaches:

- ON: `tmux [-L <socket>] -CC attach -t <id>`. This is tmux control mode. iTerm2 turns the Session into native iTerm2 windows or tabs and splits (one iTerm2 tab per Window).
- OFF: `tmux [-L <socket>] attach -t <id>`. A plain tmux client inside the current terminal.

Ctrl-T or F3 toggles it at any time and saves the choice to `settings.toml`. The Status bar always shows `iTerm2 mode: ON` or `iTerm2 mode: OFF`. How the starting value is chosen is in [usage.md](usage.md#iterm2-mode-initial-value). What `tmuxs` itself does is verified by tests; what iTerm2 does is taken from iTerm2's documentation and has not been exercised by hand (see the end of this page).

## What `-CC` is

From the tmux man page (rendered `man tmux`, tmux 3.8): `-C` starts tmux in control mode, and given twice (`-CC`) disables echo. Control mode is a text protocol (`%begin`/`%end`/`%error` blocks and `%` notifications). iTerm2 speaks that protocol. From the iTerm2 tmux integration documentation: running `tmux -CC` opens an iTerm2 window that acts like a normal iTerm2 window, and `tmux -CC attach` re-attaches. The terminal where the command ran shows a menu starting with `** tmux mode started **`, whose Esc entry detaches cleanly.

Automatically tested: the attach argument list in both modes, and that tmux 3.8 accepts an `$id` target for plain `attach`. With `-CC` and no terminal tmux fails with `tcgetattr failed` before it resolves the target, so for `-CC` only the argument parsing is proven automatically.

## Pitfall: closing versus detaching

iTerm2's documentation says that closing a session, tab or window kills the tmux session or window. In practice (per that documentation):

- Closing an iTerm2 tab, window or session in iTerm2 mode kills the matching tmux Window or Session. Work running in it is lost.
- Shell > tmux > Detach detaches and keeps the tmux Session running. All its tmux windows are closed in iTerm2 and `tmux -CC attach` (so Enter in `tmuxs`) brings them back.

This is why the kill confirmation in `tmuxs` adds the warning "iTerm2 mode is ON: this Session's iTerm2 tabs will close." Exact behavior when you close one iTerm2 window of a multi-window Session is Unverified beyond the documentation sentence.

## Hiding the control window

When `-CC` starts, the terminal tab where it ran stays as the control tab. iTerm2 has a setting named "Automatically bury the tmux client session after connecting" (Settings > General > tmux in the tmux documentation; the "tmux Integration" section of the General page). Its documented effect: when tmux integration is entered by running `tmux -CC`, the window where that command ran is buried. Enable it if you do not want the control window left visible.

`tmuxs` runs in that same window, so it is presumably buried with it. Unverified: how you get back to the picker after detaching with the setting enabled has not been tried by hand.

## Names verified from iTerm2 documentation

These were read in the iTerm2 documentation pages (https://iterm2.com/documentation-tmux-integration.html and https://iterm2.com/documentation-preferences-general.html), as recorded in `.scratch/tmuxs/research.md`:

- Menu Shell > tmux > Detach.
- Menu Shell > tmux > Dashboard.
- Setting "Automatically bury the tmux client session after connecting".
- Setting "Use "tmux" profile rather than profile of connecting session".
- Setting "When attaching, restore windows as..." (windows or tabs).
- The `** tmux mode started **` message and its command menu (`esc` detach cleanly, `X` force-quit tmux mode, `L` toggle logging, `C` run tmux command).
- The tmux integration documentation calls the settings location "Settings > General > tmux", while the General page calls it the "tmux Integration section"; the two descriptions differ.

## Not verified

- The source-level preference key of the bury setting was not located in the iTerm2 source (only that "bury" semantics exist).
- Exact menu paths in your installed iTerm2 version. The research host ran iTerm2 3.7.5; names may differ in other versions.
- Whether `TERM_PROGRAM` is `iTerm.app` inside a tmux Pane. The environment variable is documented from iTerm2 source (`PTYSession.m`) and was observed in a local iTerm2 shell, but the Pane case is Unverified.
- Whether `-CC` Attach started from a shell that is already inside tmux works. `tmuxs` refuses it before trying, so this is never exercised. The exact tmux refusal text in `-CC` mode is Unverified (plain mode prints "sessions should be nested with care, unset $TMUX to force").
- Everything in the manual checklist in [development.md](development.md#manual-verification-for-the-owner): real `-CC` attach, toggle keys inside iTerm2, appearance, and the hidden control window.
