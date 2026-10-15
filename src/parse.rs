//! Pure parsing and sorting of tmux output.

/// Format string for `tmux list-sessions -F`; fields are separated by a tab,
/// which tmux rejects in Session names, so it is a safe delimiter.
pub const SESSION_FORMAT: &str =
    "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_attached}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// tmux Session id such as `$3`; the only safe way to target a Session.
    pub id: String,
    pub name: String,
    pub windows: u32,
    pub attached: u32,
}

impl Session {
    pub fn is_attached(&self) -> bool {
        self.attached > 0
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
    if fields.next().is_some() || !id.starts_with('$') {
        return None;
    }
    Some(Session {
        id: id.to_string(),
        name: name.to_string(),
        windows,
        attached,
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
        }
    }

    #[test]
    fn parses_a_simple_line() {
        let sessions = parse_sessions("$0\talpha\t2\t1\n");
        assert_eq!(
            sessions,
            vec![Session {
                id: "$0".into(),
                name: "alpha".into(),
                windows: 2,
                attached: 1
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
            .map(|(i, n)| format!("${i}\t{n}\t1\t0\n"))
            .collect();
        let parsed: Vec<String> = parse_sessions(&output)
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(parsed, names);
    }

    #[test]
    fn accepts_an_empty_name() {
        let sessions = parse_sessions("$2\t\t1\t0\n");
        assert_eq!(sessions[0].name, "");
    }

    #[test]
    fn skips_malformed_lines() {
        let output =
            "garbage\n$1\tonly-two\n$2\tx\tNaN\t0\n$3\tx\t1\t0\textra\nid\tx\t1\t0\n$4\tok\t1\t0\n";
        let sessions = parse_sessions(output);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "$4");
    }

    #[test]
    fn parses_many_sessions() {
        let output: String = (0..500)
            .map(|i| format!("${i}\ts{i}\t{}\t0\n", i % 7 + 1))
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
}
