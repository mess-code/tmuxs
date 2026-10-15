# tmuxs research (primary sources only)

Date: 2026-10-15. Host: macOS (Darwin 27.0.0), tmux 3.8, iTerm2 3.7.5. `cargo` is NOT installed locally (`cargo --version` -> `command not found`), so crate versions come from the crates.io JSON API (https://crates.io/api/v1/crates/<name>) and docs.rs.

Notes on method: the sshs clone lived in the session scratchpad, which was removed mid-task. File paths below are repo-relative to quantumsheep/sshs at commit `f3838c07fdfb26a0456af2d2773a30bc73607943` (2026-08-29, shallow clone, no tags available, Cargo.toml version 4.8.0). Nothing from the clone was built or executed.

---

## 1. sshs

Repo: https://github.com/quantumsheep/sshs (README: "Terminal user interface for SSH. It uses `~/.ssh/config` to list and connect to hosts."). Requirement: `ssh` on PATH (README).

### Source layout (src/, 2304 lines total, via `wc -l`)
- `main.rs` (87): clap `Args`, builds `AppConfig`, `App::new(...)?.start()?`.
- `ui.rs` (689): `App`, event loop, key handling, rendering, terminal setup/restore, tests.
- `searchable.rs` (159): `Searchable<T>` generic filter using `SkimMatcherV2`.
- `ssh.rs` (310): `Host` struct, handlebars command template, `spawn_command_template`, `parse_config`.
- `ssh_config/{host,host_entry,parser,parser_error,mod}.rs`: own ssh_config parser.
- `test_support.rs`; `scripts/demo.rs` (example target).

### CLI (clap derive in `src/main.rs`; `--help` not run because no cargo, derived from source)
| Flag | Default | Doc string in source |
|---|---|---|
| `-c, --config <..>` (num_args 1..) | `/etc/ssh/ssh_config`, `~/.ssh/config` | Path to the SSH configuration file |
| `--show-proxy-command` | false | Shows ProxyCommand |
| `-s, --search <S>` | none | Host search filter |
| `--color <C>` | `blue` | Tailwind palette name |
| `--sort` | true | Sort hosts by hostname (default) |
| `--no-sort` | false | Keep config file order |
| `--sort-fancy` | false | Sort search results by fuzzy score |
| `-t, --template <T>` | `ssh "{{{name}}}"` | Handlebars template of the command to execute |
| `--on-session-start-template` | none | Command when session starts |
| `--on-session-end-template` | none | Command when session ends |
| `-e, --exit` | false | Exit after ending the SSH session |
Also `#[command(version, about)]`.

### Screen layout (`ui.rs` fn `ui`, ~L479)
`Layout::vertical([Length(3), Min(5), Length(3)])`: (1) search bar = `Paragraph` in rounded bordered `Block` with horizontal padding 3, cursor placed at `rects[0].x + cursor + 4`, `y+1`; (2) `Table` with header `Name, Aliases, User, Destination, Port` (+`Proxy` with `--show-proxy-command`), highlight symbol `" █ "` over 2 lines, `HighlightSpacing::Always`, selected row style `Modifier::REVERSED`, header color `tailwind::CYAN.c500`, border color from Tailwind palette `c400`; (3) footer, centered, text constant:
`(Esc) quit | (↑) move up | (↓) move down | (enter) select | (ctrl+r) reload`.
There is NO details/preview pane in sshs (confirmed by reading `ui()`: only three regions). tmuxs preview would be a deliberate addition.

### Key bindings (`on_key_press`, `on_key_press_ctrl`)
- Esc: quit. Ctrl+C: quit. Up/Down, Ctrl+J/Ctrl+N (next), Ctrl+K/Ctrl+P (previous), Home, End, PageUp/PageDown (step = visible rows, `page_step = max(area.height - 3, 1)`), Enter: connect, Ctrl+R: reload config.
- Everything else is forwarded to a `tui_input::Input` search box (`handle_search_event`); when search text changes, filter is re-run and selection resets to index 0.
- Only `KeyEventKind::Press` events are handled.

### Exiting the TUI and running ssh (key finding)
- NOT exec. On Enter: `restore_terminal()` -> optional start template -> `Command::new(cmd).args(args).spawn()?.wait()?` (ssh.rs L57, `println!("Running command: ...")` first) -> optional end template -> `setup_terminal()` -> loop continues unless `--exit` (then `AppKeyAction::Stop`). So the TUI returns after the child ends.
- Command line is rendered by handlebars then split with `shlex::split` (no shell).
- Terminal handling is hand-rolled, not `ratatui::init`: `setup_terminal`: `enable_raw_mode()`, `execute!(backend, Hide, EnterAlternateScreen)`; `restore_terminal`: `terminal.clear()`, `disable_raw_mode()`, `execute!(backend, Show, LeaveAlternateScreen)`. `start()` calls restore after `run` returns, then prints any error with `println!("{err:?}")`. No panic hook found (grep for "panic" only hits a test).
- Tests use `#[cfg(test)]` with App-level logic (no TestBackend usage found by grep).

### Cargo.toml (commit f3838c0)
anyhow 1.0.98; clap 4.5.37 (derive); crossterm 0.29.0; fuzzy-matcher 0.3.7 (`SkimMatcherV2`); glob 0.3.2; handlebars 6.3.2; itertools 0.14.0; ratatui 0.30.0; regex 1.11.1; serde 1.0.219 (derive); shellexpand 3.1.1; shlex 1.3.0; strum / strum_macros 0.28.0; tui-input 0.15.0; unicode-width 0.2.2. Edition 2021. Release profile: strip, lto, codegen-units=1.

### Fuzzy filtering (`searchable.rs`)
Query is split on whitespace; each token must fuzzy-match (`matcher.fuzzy_match(text, token)`) at least one of the item's search texts (name, aliases, destination); scores summed; sorted by score only with `--sort-fancy`, otherwise original order.

---

## 2. iTerm2 identification environment variables

- Source: gnachman/iTerm2 `sources/PTYSession/PTYSession.m` at commit `069b6e0dd8f3f0d82fc08ea5d294d28f7c5b986e` (master 2026-10-12), lines ~3407-3439:
  - `env[@"TERM_PROGRAM"] = @"iTerm.app"`
  - `env[@"TERM_PROGRAM_VERSION"]` = CFBundleShortVersionString
  - `env[@"TERM_SESSION_ID"]` and `env[@"ITERM_SESSION_ID"]` = session id (format `w0t0p0:<UUID>`)
  - `env[@"COLORTERM"] = @"truecolor"`
  - `env[@"ITERM_PROFILE"]` = profile name (if present)
  - `env[@"LC_TERMINAL"] = @"iTerm2"` and `LC_TERMINAL_VERSION`, only if advanced setting `shouldSetLCTerminal` is on (so can be absent)
  - `TERM_FEATURES` (not set for tmux client sessions); `TERMINFO_DIRS` only if `shouldSetTerminfoDirs`.
- Local `env | grep -i -E 'term|iterm'` output (current shell): `TERM_PROGRAM=iTerm.app`, `TERM_PROGRAM_VERSION=3.7.5`, `LC_TERMINAL=iTerm2`, `LC_TERMINAL_VERSION=3.7.5`, `ITERM_SESSION_ID=w0t0p0:A19D383C-0B0F-4745-AF29-661262D17A76`, `TERM_SESSION_ID=` same value, `ITERM_PROFILE=Default`, `__CFBundleIdentifier=com.googlecode.iterm2`, `TERM=xterm-256color`, `COLORTERM=truecolor`. **Conclusion: the current shell IS iTerm2.**
- Not verified: iterm2.com doc pages (shell-integration, variables, faq, escape-codes) did not mention these env vars in a fetch grep; the source is the authority. Note `TERM_PROGRAM` is inherited by child processes, including through ssh/tmux unless the tmux `update-environment` option is relied on (UNVERIFIED for tmux panes; inside tmux, `TERM_PROGRAM` is not guaranteed to be `iTerm.app`; tmux 3.x may set `TERM_PROGRAM=tmux`, UNVERIFIED here).
- `__CFBundleIdentifier=com.googlecode.iterm2` is set by macOS launch services (observed locally, no source cited).

---

## 3. tmux 3.8 (empirical, throwaway socket `-L research-tmp`, killed afterwards; default server never touched)

`tmux -V` -> `tmux 3.8` (`/opt/homebrew/bin/tmux` -> `../Cellar/tmux/3.8/bin/tmux`). Man page saved from `man tmux | col -b`; line refs below are into that rendering.

Gotchas for the test harness (not tmux): in zsh an unquoted `-t =alpha` is expanded by zsh's `=cmd` expansion ("alpha not found"); always quote `'=name'`. `cat` here is aliased to `bat`; use `/bin/cat`.

### Target syntax (man L420-465)
Session target resolution: `$id`, exact name, name prefix, glob. "If the session name is prefixed with an `=`, only an exact match is accepted". Window targets are `session:window`, `=` also exact.

Empirical:
```
has-session -t '=alph'   -> "can't find session: alph", rc=1
has-session -t '=alpha'  -> rc=0
has-session -t alph      -> rc=0   (prefix match, dangerous)
list-windows -t '=alpha' -F ...       -> works (session target)
list-panes   -t '=nx' / '=nx:'        -> both work
kill-session -t '=nope'  -> "can't find session: nope", rc=1
rename-session -t '=alpha' beta       -> ok
```
### capture-pane target (the important result)
`capture-pane` takes a PANE target. Empirically:
```
capture-pane -p -t '=alpha'    -> "can't find pane: =alpha"   (FAILS, also with 2 windows)
send-keys    -t '=nx' x        -> "can't find pane: =nx"      (FAILS)
capture-pane -p -t '=alpha:'   -> works (active pane of active window)
capture-pane -p -t '=alpha:.'  -> works (same content)
capture-pane -p -t 'alpha'     -> works (non-exact, prefix match)
display-message -p -t '=nx' '#{session_name}' -> printed EMPTY line, rc=0 (silently wrong)
display-message -p -t '=nx:'   -> "nx"
```
**Use `=name:` (trailing colon) for pane-targeted commands**; `-t =name` only for session-targeted commands (attach, has-session, kill-session, rename-session, list-windows, list-panes). `capture-pane -p` prints the visible region of the active pane of the active window only (no scrollback unless `-S`/`-E`). With `=alpha:` the captured text was the pane's visible screen (shell greeting), confirming active window/pane. Caveat: `=nx:` with name containing `:` fails (see sanitization below).

### list-sessions -F variables
man L3936-3966 lists: `session_activity, session_activity_flag, session_alerts, session_attached, session_attached_list, session_created, session_format, session_group, session_group_attached, session_group_attached_list, session_group_list, session_group_many_attached, session_group_size, session_grouped, session_id, session_last_attached, session_many_attached, session_marked, session_name (#S), session_path, session_stack, session_windows`.
Empirical (`tmux -L research-tmp ls -F 'name=#{session_name} win=#{session_windows} att=#{session_attached} path=#{session_path} cr=#{session_created} act=#{session_activity} id=#{session_id} grp=#{session_group} last=#{session_last_attached} alerts=#{session_alerts} stack=#{session_stack}'`):
```
name=alpha win=2 att=0 path=/tmp cr=1792081532 act=1792081532 id=$0 grp= last= alerts= stack=1,0
name=we.ird:name win=1 att=0 path=/Users/hailx/github/tmuxs cr=1792081532 act=1792081532 id=$1 grp= last= alerts= stack=0
```
`session_created`/`session_activity` are epoch seconds; `session_last_attached` empty if never attached; `session_path` is the `-c` dir. An unknown variable expands to empty (`[#{no_such_var}]` -> `[]`). `window` format: `list-windows -t '=alpha' -F '#{window_index} #{window_name} #{window_active} #{window_panes}'` -> `0 zsh 0 1` / `1 w2 1 2`.

### Sessions: create/rename/kill
- `new-session -d -s alpha -c /tmp -x 100 -y 30` ok. Duplicate: `duplicate session: beta`, rc=1.
- `-c /nonexistent` returned rc=0 and session_path became `/nonexistent` (no validation at creation).
- `kill-session -t '=beta'` ok.
- `rename-session -t '=nx' "a<TAB>b"` -> `invalid session name: a	b`, rc=1; same for `new-session -s "tab<TAB>x"`.
- Names containing `.` and `:` ARE accepted verbatim in 3.8 (`we.ird:name`, `a:b.c`, `q:r.s` all listed as-is, no sanitization to `_`), contrary to older tmux behavior (UNVERIFIED for older versions; no man text on sanitization found). But they cannot be targeted: `capture-pane -t '=we.ird:name'` -> `can't find session: we.ird`; `-t '=q:r.s:'` -> `can't find session: q`. Prefer `$id` (`session_id`) as the stable target for such names.
- `new-session -d -s ''` returned rc=0 and created a session whose name is the empty string (listed as `[]`, id `$2`). Edge case to defend against when parsing.
- Delimiter: tab is rejected in names, so `\t` is a safe field separator in `-F` output (verified rejection on create and rename). Newline not tested (UNVERIFIED).

### Nested attach refusal
Run from a shell inside a pane of the same (throwaway) server:
```
tmux -L research-tmp attach -t "=nx"            -> "sessions should be nested with care, unset $TMUX to force", RC=1
tmux -L research-tmp new-session -s zz          -> same message, RC=1
tmux -L research-tmp new-session -d -s zz2      -> RC=0 (detached create is fine)
tmux -L research-tmp -CC attach -t "=nx"        -> RC=1 (message not captured; UNVERIFIED reason)
env -u TMUX tmux -L research-tmp new-session -d -s zz3 -> RC=0
```
Outside a pane, merely exporting a fake `TMUX=/tmp/x,1,0` did NOT trigger the refusal (attach under `script` pty proceeded to draw the UI): the check apparently fires only when the client tty is a pane of that server (tmux source `server_client_check_nested`, from memory; UNVERIFIED, source not read). `man` (L598-): attach "If used from inside, switch the currently attached session". So inside tmux the right action is `switch-client -t =name` (needs a client; outside any client: "no current client"); from outside use `attach`. Without a pty: `open terminal failed: not a terminal`.

### -CC
Man L44-45: "-C Start in control mode ... Given twice (-CC) disables echo." CONTROL MODE section (L4790+): text protocol with `%begin/%end/%error` blocks and `%` notifications. `tmux -CC attach -t =name` needs a real terminal (iTerm2 provides it). Not run for real here (needs iTerm2 UI).

### -L
Socket under `/private/tmp/tmux-501/<name>` (message: "no server running on /private/tmp/tmux-501/research-tmp"). `tmux -L research-tmp kill-server` run at the end of each test; afterwards `ls` reports no server.

---

## 4. iTerm2 tmux integration

Sources: https://iterm2.com/documentation-tmux-integration.html (fetched), https://iterm2.com/documentation-preferences-general.html (fetched).
- What -CC does: "When you run `tmux -CC`, a new tmux session is created ... An iTerm2 window opens and it acts like a normal iTerm2 window." `tmux -CC attach` re-attaches; windows "reopen in the same state". The terminal where it ran shows a menu: "`** tmux mode started **` Command Menu: `esc` Detach cleanly. `X` Force-quit tmux mode. `L` Toggle logging. `C` Run tmux command." "If you press esc, the tmux windows will close and the tmux client will exit."
- Detach: menu **Shell > tmux > Detach** ("Detaches from the tmux session. All tmux windows are closed. You can get them back with `tmux -CC attach`").
- Closing: "Close a session, tab, or window: Kills the tmux session or window." (i.e. closing an iTerm2 tab kills the tmux window, and closing the last one ends the session; detach is the non-destructive path). Exact semantics when closing the window of a multi-window session: UNVERIFIED beyond that sentence.
- Settings page (General > tmux, called "Settings > General > tmux" in the tmux doc, "tmux Integration section" on the General page): **"Automatically bury the tmux client session after connecting"**: "When the tmux integration is entered by running `tmux -CC`, the window in which that command was run will be buried." Also **"Use "tmux" profile rather than profile of connecting session"** (not default since 3.3) and **"When attaching, restore windows as…"** (windows vs tabs; pref key `OpenTmuxWindowsIn`, `iTermPreferences.m` L82, L753).
- 3.7 note (tmux doc): advanced setting **"Open tmux windows and tabs in the background?"** keeps keyboard focus on the current session.
- Menu: **Shell > tmux > Dashboard**; "Shell->tmux menu" creates a new tmux window.
- Source check: `PTYSession.m` has `- (void)hideSession { [self bury]; }` and `TmuxController.m` L2878 logs "burying window IDs", consistent with "bury" semantics; the bury-on-connect preference key name was not located (UNVERIFIED).
- Not verified: whether a `-CC` attach from a window that is itself already inside a tmux pane works (no; tmux nested refusal applies per section 3).

---

## 5. Rust crates (crates.io API, 2026-10-15)

| Crate | Latest | Notes |
|---|---|---|
| ratatui | 0.30.2 (2026-06-19), MSRV 1.88.0 (0.30.0 was 1.86.0) | default features: all-widgets, crossterm, layout-cache, macros, underline-color (docs.rs feature page) |
| crossterm | 0.29.0 (2025-04-05), MSRV 1.63.0 | ratatui 0.30.2 dev-dep `crossterm ^0.29`; backend crate `ratatui-crossterm ^0.1.2` |
| ratatui-crossterm | ^0.1.2 | optional dep |
| nucleo-matcher | 0.3.1 (2024-02-20) | |
| nucleo | 0.5.0 (2024-04-02) | |
| fuzzy-matcher | 0.3.7 (2020-10-04) | what sshs uses; unmaintained since 2020 |
| toml | 1.1.8+spec-1.1.0 (2026-10-09) | |
| serde | 1.0.229 | |
| dirs | 7.0.0 (2026-09-05) | |
| directories | 6.0.0 (2025-01-12) | |
| etcetera | 0.11.0 (2025-10-28) | |
| tui-input | 0.15.5 | sshs uses 0.15.0 |
| clap | 4.6.7 | sshs uses 4.5.37 |
| anyhow | 1.0.104 | |

- Does ratatui re-export crossterm? YES under feature `crossterm` (default): `pub use ratatui_crossterm::crossterm;` (ratatui/src/lib.rs on main, L487-489, comment "re-export the `crossterm` crate so that users don't have to add it as a dependency"). The docs quickstart still says `cargo add ratatui crossterm`. Which crossterm version the re-export tracks (features `crossterm_0_28`/`crossterm_0_29` exist): default selection UNVERIFIED; adding `crossterm = "0.29"` directly matches sshs, otherwise use `ratatui::crossterm::event` to avoid version skew.
- Helpers (docs.rs 0.30.2): `ratatui::init() -> DefaultTerminal`: crossterm backend on Stdout, raw mode on, alternate screen on, **installs a panic hook that restores the terminal** (call after any other panic hooks); panics if setup fails; `try_init` for fallible variant. `ratatui::restore()`: disables raw mode, leaves alternate screen; errors printed to stderr and ignored; `try_restore` fallible. `ratatui::run(|terminal| ...)` (new in 0.30) = init + closure + restore. `init_with_options` for custom `Viewport`. Does not hide/show cursor explicitly (docs list only raw mode and alt screen). To run a child (ssh/tmux attach) from the TUI: call `restore()`, run child, call `init()` again (note: each `init()` installs another panic hook, UNVERIFIED impact; sshs avoids by hand-rolled setup).
- TestBackend (docs.rs `ratatui::backend::TestBackend`): `TestBackend::new(w, h)`, `with_lines(...)`, `buffer(&self) -> &Buffer`, `scrollback()`, `resize`, `cursor_position()`, `cursor_visible()`, `assert_buffer(&Buffer)`, `assert_scrollback`, `assert_buffer_lines([...])` (shown in the docs example). Typical use: `let mut t = Terminal::new(TestBackend::new(80,24))?; t.draw(|f| ...)?; t.backend().buffer()`; `Buffer` is at `ratatui::buffer::Buffer`. Docs advise unit-testing widgets directly against `Buffer` and reserving TestBackend for whole-UI tests. Whether a top-level `assert_buffer_eq!` macro still exists in 0.30: NOT FOUND in docs.rs all-items list (grep for `assert_buffer`), treat as removed/UNVERIFIED; use `assert_eq!(buffer, &expected)` or `backend.assert_buffer_lines`.

### Recommendations
- **Filtering**: for a list of tens of sessions with short names, recommend plain case-insensitive multi-token substring filtering first (no dependency, predictable, easy to test). If fuzzy is wanted, use `nucleo-matcher` 0.3.1 (same author as helix; Pattern API supports multiple atoms, active relative to fuzzy-matcher which is unmaintained since 2020), avoiding the heavier `nucleo` 0.5.0 (threaded worker, built for huge lists). sshs uses `fuzzy-matcher` 0.3.7 with per-token matching; imitating its token semantics (every whitespace token must match some field) is cheap either way.
- **Config**: `serde` 1.0 (derive) + `toml` 1.1 for `config.toml`. Paths: `dirs` 7.0.0 returns macOS `~/Library/Application Support` for `config_dir()`, not XDG; for a CLI tool wanting `~/.config/tmuxs/config.toml` on macOS use `etcetera` 0.11 (`choose_base_strategy()` gives XDG on macOS), or honor `$XDG_CONFIG_HOME` manually. (dirs macOS behavior: from its README, recalled, UNVERIFIED here.)
- ratatui 0.30.2 needs rustc >= 1.88; install a toolchain via rustup (cargo currently absent).

---

## Implications for tmuxs

1. capture-pane/preview must use `-t '=name:'` (or `$session_id:`), not `-t =name`; keep `=name` only for session-level commands. Always quote `=` when scripting under zsh. Prefer `session_id` (`$N`) as the internal key; names with `.`/`:` are accepted by tmux 3.8 but unreachable via `=name`, and empty names are possible.
2. Listing: `tmux ls -F` with tab as separator (tab is rejected in session names), fields `session_id, session_name, session_windows, session_attached, session_path, session_created, session_activity` all exist in 3.8. Handle "no server running" (rc=1) as an empty list.
3. Attach flow: if `$TMUX` is set and the client is in a pane, use `switch-client -t =name`; plain `attach` is refused with "sessions should be nested with care". For iTerm2 use `tmux -CC attach -t =name`; detect iTerm2 via `TERM_PROGRAM=iTerm.app` (corroborated by `LC_TERMINAL=iTerm2`, which may be absent depending on an advanced setting; `ITERM_SESSION_ID` also present). Current shell is iTerm2 3.7.5.
4. Mimic sshs: three-region vertical layout (search 3 rows / table / footer 3 rows), Esc/Ctrl+C quit, Up/Down, Ctrl+J/K/N/P, Home/End/PgUp/PgDn, Enter, Ctrl+R reload; whitespace-token filter that resets selection to row 0 on query change; restore terminal, spawn child and wait, re-enter TUI (`--exit`-style flag to quit instead). A details/preview pane is NOT in sshs; adding it is a tmuxs design choice (capture-pane preview).
5. Terminal handling: prefer `ratatui::init/restore` (panic hook included) over sshs's hand-rolled setup; test via `Terminal<TestBackend>` + `backend().buffer()`.
6. -CC behavior notes for UX: closing an iTerm2 tab/window kills the tmux window/session; Shell > tmux > Detach is the non-destructive route; "Automatically bury the tmux client session after connecting" controls what happens to the launching tab. Warn users in docs.
7. UNVERIFIED list: `-CC` nested refusal message text; `server_client_check_nested` implementation; `TERM_PROGRAM` value inside tmux panes; default crossterm version behind `ratatui::crossterm`; existence of `assert_buffer_eq!` in 0.30; dirs macOS behavior; bury setting's source-level key; newline in session names; sshs `--help` actual output (cargo unavailable).
