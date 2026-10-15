# tmuxs

A terminal picker for tmux sessions, in the spirit of `sshs` for SSH hosts. It lists sessions and lets the user attach to, rename, kill and create them, so long-lived agent work can be kept in tmux without any other GUI tool.

## Language

**Session**:
A tmux session, identified by its exact name.
_Avoid_: Workspace, project, host

**Attach**:
Handing the terminal over to tmux to work inside a Session, ending `tmuxs` itself.
_Avoid_: Connect, open, enter

**iTerm2 mode**:
A user setting that decides whether Attach uses tmux control mode (`-CC`, rendered by iTerm2 as native tabs) or a plain tmux client.
_Avoid_: CC mode, control mode toggle

**Window**:
A tmux window inside a Session. In iTerm2 mode it opens as one iTerm2 tab.
_Avoid_: Tab, terminal

**Pane**:
A single terminal inside a Window. Panes are what `tmuxs` counts as a Session's terminals.
_Avoid_: Terminal, split

**Pane state**:
Whether a Pane is *busy* (its foreground command is not a shell), *idle* (its foreground command is a shell) or *dead* (its process has exited).
_Avoid_: Live/living, status

**Search query**:
The text typed into the always-focused search bar that narrows the visible Sessions by name.
_Avoid_: Filter string, search mode

**Activity age**:
How long ago a Session last had activity, shown as a short relative duration (`5s`, `12m`, `3h`, `2d`).
_Avoid_: Last seen, timestamp

**Popup**:
A centered dialog for rename, create or kill confirmation. A failed rename or create keeps the Popup open and shows the error inside it.
_Avoid_: Modal, prompt

**Status bar**:
The bottom line showing key hints, the current iTerm2 mode, and result messages.
_Avoid_: Footer, toast
