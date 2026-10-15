# Key bindings

This page lists every binding implemented in `App::handle_key` (`src/app.rs`) and `Popup::cancels` / `Popup::handle_key` (`src/popup.rs`). Only key presses are handled; release and repeat-release events are ignored (`KeyEventKind::Press`).

The Status bar hints show only a subset: `(Esc) quit | (↑/↓) move | (Enter) attach | (^N) new | (^R) rename | (^X) kill | (^T) iTerm2 mode | type to search`. F2, F3, Delete and Ctrl-J/Ctrl-K are not in the hints. Any key press clears a result message, which brings the hints back.

## Main screen (no Popup open)

The search bar is always focused, so every plain printable key edits the Search query. Actions use Ctrl or function keys.

| Key | Action |
|---|---|
| Printable character (no Ctrl, no Alt) | Append to the Search query and re-filter |
| Backspace | Remove the last character of the Search query |
| Up / Ctrl-K | Move the selection up (stops at the first row) |
| Down / Ctrl-J | Move the selection down (stops at the last row) |
| Enter | Attach to the selected Session (refused inside tmux) |
| Ctrl-N | Open the create Popup (works even with no Sessions) |
| Ctrl-R or F2 | Open the rename Popup for the selected Session |
| Ctrl-X or Delete | Open the kill confirmation for the selected Session |
| Ctrl-T or F3 | Toggle iTerm2 mode and save it |
| Esc or Ctrl-C | Quit |

Notes:

- Rename and kill do nothing when no Session is selected (empty table). Enter does nothing then too.
- Any other key, including Ctrl chords not listed, Alt chords, Tab, Left, Right, Home, End, PageUp and PageDown, is ignored. Home/End/PageUp/PageDown exist in `sshs` but are not implemented here.
- The Search query is append-only: there is no cursor movement, no word delete and no clear key. Backspace is the only editing key.

## Popups

While a Popup is open it receives all keys; main-screen bindings do not apply. Esc and Ctrl-C close the Popup and do not quit the program. A second Esc or Ctrl-C (with no Popup) quits.

### Rename Popup

| Key | Action |
|---|---|
| Printable character (no Ctrl, no Alt) | Append to the name |
| Backspace | Remove the last character |
| Enter | Submit. An empty or whitespace-only name shows "The name cannot be empty." and keeps the Popup open. A tmux error is shown inside the Popup. |
| Esc or Ctrl-C | Cancel |

Typing or Backspace clears an error shown in the Popup. The name is trimmed before it is sent to tmux.

### Create Popup

Fields: Name (focused first, prefilled from the Search query) and Directory (prefilled with the launch directory).

| Key | Action |
|---|---|
| Tab, Shift-Tab, Up or Down | Switch between Name and Directory (with two fields all four just toggle) |
| Printable character (no Ctrl, no Alt) | Append to the focused field |
| Backspace | Remove the last character of the focused field |
| Enter | Submit from either field. An empty name shows the empty-name error and moves focus to Name. |
| Esc or Ctrl-C | Cancel |

### Kill confirmation

| Key | Action |
|---|---|
| y or Y | Kill the Session |
| n, N, Enter, Esc or Ctrl-C | Cancel (the default is No) |
| Any other key | Ignored; the Popup stays open |

## Known gaps

- Text fields (Search query, Popup fields) are append-only. You cannot move the cursor or edit the middle of the text. The cursor is drawn at the end of the text.
- F3 may be intercepted by the terminal or macOS (for example Mission Control on some keyboards) before it reaches `tmuxs`. Unverified in a real iTerm2; Ctrl-T always works as an alternative. Likewise F2 is untested in a real terminal.
- Popup layout on very small terminals is untested. The Popup is at most 64 columns wide and its height is capped to the terminal height, so on a small terminal lines may be cut off.
- The Status bar line is one unwrapped line, so on a narrow terminal the hints and the iTerm2 mode label are cut off (inferred from the code, not observed).
- Bracketed paste is not enabled; pasted text arrives as individual key events. Unverified.
