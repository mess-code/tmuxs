# Records docs/images/demo.gif. Run from the repository root:
#   cargo build --release && vhs docs/demo.tape
# The demo uses a throwaway tmux server and settings directory, so it never
# touches real Sessions or settings.

Output docs/images/demo.gif

Set Shell "zsh"
Set FontSize 16
Set Width 1200
Set Height 560
Set Padding 16
Set Theme "Catppuccin Mocha"
Set TypingSpeed 90ms
Set Framerate 20

Hide
Type "export PATH=$PWD/target/release:$PATH TMUXS_SOCKET=tmuxs-demo TMUXS_CONFIG_DIR=$(mktemp -d) TMUXS_CC=0 PS1='$ '"
Enter
Type "mkdir -p /tmp/tmuxs-demo/{agent,api,docs,notes} && tmux -L tmuxs-demo kill-server 2>/dev/null; true"
Enter
Type "tmux -L tmuxs-demo new-session -d -s agent-claude -c /tmp/tmuxs-demo/agent 'sleep 3600' && tmux -L tmuxs-demo split-window -t agent-claude: -c /tmp/tmuxs-demo/agent && tmux -L tmuxs-demo new-session -d -s api-server -c /tmp/tmuxs-demo/api 'sleep 3600' && tmux -L tmuxs-demo new-session -d -s docs -c /tmp/tmuxs-demo/docs && tmux -L tmuxs-demo new-session -d -s notes -c /tmp/tmuxs-demo/notes"
Enter
Type "cd /tmp/tmuxs-demo && clear"
Enter
Show

Type "tmuxs"
Sleep 400ms
Enter
Sleep 2s

# Search
Type "a"
Sleep 800ms
Type "gent"
Sleep 1500ms
Backspace 5
Sleep 800ms

# Move
Down
Sleep 600ms
Down
Sleep 800ms

# Create
Ctrl+N
Sleep 1s
Type "demo-feature"
Sleep 600ms
Enter
Sleep 1500ms

# Rename
Ctrl+R
Sleep 800ms
Type "-v2"
Sleep 600ms
Enter
Sleep 1500ms

# Kill, with confirmation
Ctrl+X
Sleep 1800ms
Type "y"
Sleep 1500ms

# Toggle iTerm2 mode
Ctrl+T
Sleep 1500ms
Ctrl+T
Sleep 1200ms

# Attach with a plain tmux client, then detach and come back
Enter
Sleep 2s
Ctrl+B
Type "d"
Sleep 2s

Escape
Sleep 500ms

Hide
Type "tmux -L tmuxs-demo kill-server; rm -rf /tmp/tmuxs-demo"
Enter
Sleep 300ms
