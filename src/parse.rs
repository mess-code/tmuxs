//! Pure parsing and sorting of tmux output.

/// Format string for `tmux list-sessions -F`; fields are separated by a tab,
/// which tmux rejects in Session names, so it is a safe delimiter.
/// `session_path` expands to an empty string on a tmux that does not know it.
pub const SESSION_FORMAT: &str = "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_attached}\t#{session_activity}\t#{session_path}";

/// Format string for `tmux list-panes -a -F`.
pub const PANE_FORMAT: &str =
    "#{session_id}\t#{window_index}\t#{window_name}\t#{pane_dead}\t#{pane_current_command}";

/// Foreground commands that count as a shell sitting at its prompt.
const SHELLS: [&str; 8] = ["sh", "bash", "zsh", "fish", "dash", "ksh", "tcsh", "nu"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    /// tmux Session id such as `$3`; the only safe way to target a Session.
    pub id: String,
    pub name: String,
    pub windows: u32,
    pub attached: u32,
    /// Last activity, epoch seconds.
    pub activity: u64,
    /// Working directory; empty when the installed tmux does not support it.
    pub path: String,
    pub panes: Vec<Pane>,
}

impl Session {
    pub fn is_attached(&self) -> bool {
        self.attached > 0
    }

    /// Number of Panes in each Pane state. `shell` is the basename of `$SHELL`.
    pub fn pane_counts(&self, shell: Option<&str>) -> PaneCounts {
        let mut counts = PaneCounts::default();
        for pane in &self.panes {
            match pane_state(pane, shell) {
                PaneState::Busy => counts.busy += 1,
                PaneState::Idle => counts.idle += 1,
                PaneState::Dead => counts.dead += 1,
            }
        }
        counts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub session_id: String,
    pub window_index: u32,
    pub window_name: String,
    pub dead: bool,
    /// Foreground command of the Pane.
    pub command: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneState {
    Busy,
    Idle,
    Dead,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PaneCounts {
    pub busy: u32,
    pub idle: u32,
    pub dead: u32,
}

/// Dead if the process exited, idle if the foreground command is a shell
/// (fixed list or `shell`, the basename of `$SHELL`), busy otherwise.
/// A heuristic: an agent waiting for input still counts as busy.
pub fn pane_state(pane: &Pane, shell: Option<&str>) -> PaneState {
    // Login shells can be reported as `-zsh`.
    let command = pane.command.trim_start_matches('-');
    if pane.dead {
        PaneState::Dead
    } else if SHELLS.contains(&command) || shell == Some(command) {
        PaneState::Idle
    } else {
        PaneState::Busy
    }
}

/// The basename of a `$SHELL` value, or `None` when it is empty.
pub fn shell_basename(shell: &str) -> Option<String> {
    let name = shell.rsplit('/').next()?;
    (!name.is_empty()).then(|| name.to_string())
}

/// One-line summary of a Session's Windows, such as `0:zsh 1:claude`.
pub fn window_summary(session: &Session) -> String {
    let mut windows: Vec<(u32, &str)> = session
        .panes
        .iter()
        .map(|p| (p.window_index, p.window_name.as_str()))
        .collect();
    windows.sort();
    windows.dedup_by_key(|w| w.0);
    let parts: Vec<String> = windows.iter().map(|(i, n)| format!("{i}:{n}")).collect();
    parts.join(" ")
}

/// Short relative age such as `5s`, `12m`, `3h` or `2d`.
pub fn format_age(now: u64, activity: u64) -> String {
    let secs = now.saturating_sub(activity);
    match secs {
        0..60 => format!("{secs}s"),
        60..3600 => format!("{}m", secs / 60),
        3600..86400 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86400),
    }
}

/// Case-insensitive substring match of the Search query against a Session name.
pub fn matches_query(name: &str, query: &str) -> bool {
    name.to_lowercase().contains(&query.to_lowercase())
}

/// Parses `list-panes -a -F PANE_FORMAT` output. Malformed lines are skipped.
pub fn parse_panes(output: &str) -> Vec<Pane> {
    output.lines().filter_map(parse_pane_line).collect()
}

fn parse_pane_line(line: &str) -> Option<Pane> {
    let fields: Vec<&str> = line.split('\t').collect();
    // Anything between the index and the last two fields belongs to the window name.
    let [id, index, name @ .., dead, command] = fields.as_slice() else {
        return None;
    };
    if name.is_empty() || !id.starts_with('$') {
        return None;
    }
    Some(Pane {
        session_id: id.to_string(),
        window_index: index.parse().ok()?,
        window_name: name.join("\t"),
        dead: match *dead {
            "0" => false,
            "1" => true,
            _ => return None,
        },
        command: command.to_string(),
    })
}

/// Moves each Pane into the Session with the same id.
pub fn assign_panes(sessions: &mut [Session], panes: Vec<Pane>) {
    for pane in panes {
        if let Some(session) = sessions.iter_mut().find(|s| s.id == pane.session_id) {
            session.panes.push(pane);
        }
    }
}

/// Parses `list-sessions -F SESSION_FORMAT` output. Malformed lines are skipped.
pub fn parse_sessions(output: &str) -> Vec<Session> {
    output.lines().filter_map(parse_session_line).collect()
}

fn parse_session_line(line: &str) -> Option<Session> {
    let mut fields = line.split('\t');
    let id = fields.next()?;
    let name = fields.next()?;
    let windows = fields.next()?.parse().ok()?;
    let attached = fields.next()?.parse().ok()?;
    let activity = fields.next()?.parse().ok()?;
    let path = fields.next()?;
    if fields.next().is_some() || !id.starts_with('$') {
        return None;
    }
    Some(Session {
        id: id.to_string(),
        name: name.to_string(),
        windows,
        attached,
        activity,
        path: path.to_string(),
        panes: Vec::new(),
    })
}

/// Sorts Sessions by name, then by id so the order is fully deterministic.
pub fn sort_sessions(sessions: &mut [Session]) {
    sessions.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str, name: &str) -> Session {
        Session {
            id: id.into(),
            name: name.into(),
            windows: 1,
            attached: 0,
            ..Session::default()
        }
    }

    #[test]
    fn parses_a_simple_line() {
        let sessions = parse_sessions("$0\talpha\t2\t1\t0\t\n");
        assert_eq!(
            sessions,
            vec![Session {
                id: "$0".into(),
                name: "alpha".into(),
                windows: 2,
                attached: 1,
                ..Session::default()
            }]
        );
        assert!(sessions[0].is_attached());
    }

    #[test]
    fn keeps_unusual_names_verbatim() {
        let names = [
            "my work",
            "we.ird:name",
            "a:b.c",
            "  padded  ",
            "ünï 🚀",
            "$dollar",
            "-dash",
        ];
        let output: String = names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("${i}\t{n}\t1\t0\t0\t/\n"))
            .collect();
        let parsed: Vec<String> = parse_sessions(&output)
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(parsed, names);
    }

    #[test]
    fn accepts_an_empty_name() {
        let sessions = parse_sessions("$2\t\t1\t0\t0\t\n");
        assert_eq!(sessions[0].name, "");
    }

    #[test]
    fn skips_malformed_lines() {
        let output = "garbage\n$1\tonly-two\n$2\tx\tNaN\t0\t0\t\n$3\tx\t1\t0\t0\t\textra\nid\tx\t1\t0\t0\t\n$4\tok\t1\t0\t0\t\n";
        let sessions = parse_sessions(output);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "$4");
    }

    #[test]
    fn parses_many_sessions() {
        let output: String = (0..500)
            .map(|i| format!("${i}\ts{i}\t{}\t0\t0\t\n", i % 7 + 1))
            .collect();
        let sessions = parse_sessions(&output);
        assert_eq!(sessions.len(), 500);
        assert_eq!(sessions[499].id, "$499");
        assert_eq!(sessions[499].windows, 499 % 7 + 1);
    }

    #[test]
    fn empty_output_is_no_sessions() {
        assert!(parse_sessions("").is_empty());
    }

    #[test]
    fn sorts_by_name_and_keeps_cat_and_category_distinct() {
        let mut sessions = vec![
            session("$2", "category"),
            session("$9", "b"),
            session("$1", "cat"),
            session("$5", "b"),
        ];
        sort_sessions(&mut sessions);
        let order: Vec<_> = sessions.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(order, ["$5", "$9", "$1", "$2"]);
    }

    #[test]
    fn parses_activity_and_path() {
        let s = &parse_sessions("$0\ta\t1\t0\t1700000000\t/tmp/my dir\n")[0];
        assert_eq!(s.activity, 1_700_000_000);
        assert_eq!(s.path, "/tmp/my dir");
    }

    #[test]
    fn an_empty_path_means_the_variable_is_unsupported() {
        let s = &parse_sessions("$0\ta\t1\t0\t5\t\n")[0];
        assert_eq!(s.path, "");
    }

    fn pane(command: &str, dead: bool) -> Pane {
        Pane {
            session_id: "$0".into(),
            window_index: 0,
            window_name: "w".into(),
            dead,
            command: command.into(),
        }
    }

    #[test]
    fn parses_panes_with_unusual_window_names() {
        let output = "$0\t0\tzsh\t0\tzsh\n$0\t1\tmy win: 1.2\t1\tclaude\n$3\t2\ta\tb\t0\tvim\n";
        let panes = parse_panes(output);
        assert_eq!(panes.len(), 3);
        assert_eq!(panes[1].window_name, "my win: 1.2");
        assert!(panes[1].dead);
        assert_eq!(panes[1].command, "claude");
        assert_eq!(panes[2].window_name, "a\tb");
        assert_eq!(panes[2].session_id, "$3");
    }

    #[test]
    fn skips_malformed_pane_lines() {
        assert!(parse_panes("junk\n$0\tx\tn\t0\tzsh\n$0\t0\tn\t2\tzsh\n").is_empty());
    }

    #[test]
    fn pane_state_is_dead_idle_or_busy() {
        assert_eq!(pane_state(&pane("zsh", true), None), PaneState::Dead);
        assert_eq!(pane_state(&pane("zsh", false), None), PaneState::Idle);
        for shell in ["sh", "bash", "zsh", "fish", "dash", "ksh", "tcsh", "nu"] {
            assert_eq!(pane_state(&pane(shell, false), None), PaneState::Idle);
        }
        assert_eq!(pane_state(&pane("-zsh", false), None), PaneState::Idle);
        assert_eq!(pane_state(&pane("claude", false), None), PaneState::Busy);
        assert_eq!(pane_state(&pane("xonsh", false), None), PaneState::Busy);
        assert_eq!(
            pane_state(&pane("xonsh", false), Some("xonsh")),
            PaneState::Idle
        );
    }

    #[test]
    fn shell_basename_comes_from_the_path() {
        assert_eq!(
            shell_basename("/opt/homebrew/bin/xonsh"),
            Some("xonsh".into())
        );
        assert_eq!(shell_basename(""), None);
    }

    #[test]
    fn counts_panes_by_state_and_summarizes_windows() {
        let mut s = session("$0", "a");
        s.panes = vec![
            pane("zsh", false),
            pane("sleep", false),
            pane("sleep", false),
            pane("zsh", true),
        ];
        s.panes[1].window_index = 1;
        s.panes[1].window_name = "claude".into();
        s.panes[2].window_index = 1;
        s.panes[2].window_name = "claude".into();
        s.panes[0].window_name = "zsh".into();
        s.panes[3].window_name = "zsh".into();
        assert_eq!(
            s.pane_counts(None),
            PaneCounts {
                busy: 2,
                idle: 1,
                dead: 1
            }
        );
        assert_eq!(window_summary(&s), "0:zsh 1:claude");
        assert_eq!(window_summary(&session("$1", "b")), "");
    }

    #[test]
    fn assigns_panes_to_their_sessions_by_id() {
        let mut sessions = vec![session("$0", "a"), session("$1", "b")];
        let mut p = pane("zsh", false);
        p.session_id = "$1".into();
        assign_panes(&mut sessions, vec![p, pane("vim", false)]);
        assert_eq!(sessions[0].panes.len(), 1);
        assert_eq!(sessions[1].panes[0].command, "zsh");
    }

    #[test]
    fn formats_activity_age() {
        let age = |secs: u64| format_age(1_000_000, 1_000_000 - secs);
        assert_eq!(age(0), "0s");
        assert_eq!(age(5), "5s");
        assert_eq!(age(59), "59s");
        assert_eq!(age(60), "1m");
        assert_eq!(age(12 * 60 + 30), "12m");
        assert_eq!(age(3 * 3600), "3h");
        assert_eq!(age(2 * 86400 + 5), "2d");
        assert_eq!(format_age(10, 20), "0s");
    }

    #[test]
    fn filter_is_a_case_insensitive_substring_match() {
        assert!(matches_query("Category", "cat"));
        assert!(matches_query("my WORK", "work"));
        assert!(matches_query("anything", ""));
        assert!(matches_query("we.ird:name", "d:n"));
        assert!(!matches_query("dog", "cat"));
        assert!(!matches_query("cat", "category"));
    }
}
