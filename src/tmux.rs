//! Runs tmux through `std::process::Command`.

use crate::parse::{
    PANE_FORMAT, SESSION_FORMAT, Session, assign_panes, parse_panes, parse_sessions, sort_sessions,
};
use std::fmt;
use std::io::ErrorKind;
use std::process::Command;

/// Environment variable that makes every tmux call use `tmux -L <value>`.
pub const SOCKET_ENV: &str = "TMUXS_SOCKET";

#[derive(Debug, PartialEq, Eq)]
pub enum TmuxError {
    NotInstalled,
    Failed(String),
}

impl fmt::Display for TmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TmuxError::NotInstalled => {
                write!(
                    f,
                    "tmux was not found on your PATH. Install tmux and try again."
                )
            }
            TmuxError::Failed(message) => write!(f, "tmux failed: {message}"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tmux {
    socket: Option<String>,
}

impl Tmux {
    pub fn new(socket: Option<String>) -> Self {
        Self {
            socket: socket.filter(|s| !s.is_empty()),
        }
    }

    pub fn from_env() -> Self {
        Self::new(std::env::var(SOCKET_ENV).ok())
    }

    pub fn socket(&self) -> Option<&str> {
        self.socket.as_deref()
    }

    /// A `tmux` command, already pointed at the configured socket.
    pub fn command(&self) -> Command {
        let mut command = Command::new("tmux");
        if let Some(socket) = &self.socket {
            command.arg("-L").arg(socket);
        }
        command
    }

    /// Lists all Sessions sorted by name. No server means no Sessions.
    pub fn list_sessions(&self) -> Result<Vec<Session>, TmuxError> {
        let output = self
            .command()
            .args(["list-sessions", "-F", SESSION_FORMAT])
            .output()
            .map_err(|e| match e.kind() {
                ErrorKind::NotFound => TmuxError::NotInstalled,
                _ => TmuxError::Failed(e.to_string()),
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return if is_no_server(&stderr) {
                Ok(Vec::new())
            } else {
                Err(TmuxError::Failed(stderr.trim().to_string()))
            };
        }
        let mut sessions = parse_sessions(&String::from_utf8_lossy(&output.stdout));
        sort_sessions(&mut sessions);
        assign_panes(&mut sessions, self.list_panes());
        Ok(sessions)
    }

    /// Creates a detached Session and returns its `$id`. An empty `dir` means
    /// tmux's default. tmux accepts a missing directory, so it is checked here.
    pub fn create_session(&self, name: &str, dir: &str) -> Result<String, TmuxError> {
        let mut args = vec!["new-session", "-d", "-P", "-F", "#{session_id}", "-s", name];
        let dir = expand_tilde(dir, std::env::var("HOME").ok().as_deref());
        if !dir.is_empty() {
            if !std::path::Path::new(&dir).is_dir() {
                return Err(TmuxError::Failed(format!(
                    "directory does not exist: {dir}"
                )));
            }
            args.extend(["-c", &dir]);
        }
        self.run(&args).map(|id| id.trim().to_string())
    }

    pub fn rename_session(&self, id: &str, name: &str) -> Result<(), TmuxError> {
        // `--` keeps a name that starts with `-` from being read as a flag.
        self.run(&["rename-session", "-t", id, "--", name])
            .map(drop)
    }

    pub fn kill_session(&self, id: &str) -> Result<(), TmuxError> {
        self.run(&["kill-session", "-t", id]).map(drop)
    }

    /// Runs one tmux command and returns its stdout.
    fn run(&self, args: &[&str]) -> Result<String, TmuxError> {
        let output = self
            .command()
            .args(args)
            .output()
            .map_err(|e| match e.kind() {
                ErrorKind::NotFound => TmuxError::NotInstalled,
                _ => TmuxError::Failed(e.to_string()),
            })?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(TmuxError::Failed(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    /// Lists the Panes of every Session. Any failure means no Pane data.
    fn list_panes(&self) -> Vec<crate::parse::Pane> {
        self.command()
            .args(["list-panes", "-a", "-F", PANE_FORMAT])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| parse_panes(&String::from_utf8_lossy(&output.stdout)))
            .unwrap_or_default()
    }
}

/// Expands a leading `~` or `~/` to the home directory, when known.
pub fn expand_tilde(dir: &str, home: Option<&str>) -> String {
    match (home, dir.strip_prefix('~')) {
        (Some(home), Some(rest)) if rest.is_empty() || rest.starts_with('/') => {
            format!("{}{rest}", home.trim_end_matches('/'))
        }
        _ => dir.to_string(),
    }
}

fn is_no_server(stderr: &str) -> bool {
    stderr.contains("no server running") || stderr.contains("error connecting to")
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn detects_missing_server_messages() {
        assert!(is_no_server("no server running on /private/tmp/tmux-501/x"));
        assert!(is_no_server(
            "error connecting to /tmp/tmux-1/default (No such file or directory)"
        ));
        assert!(!is_no_server("protocol version mismatch"));
    }

    #[test]
    fn command_honors_the_socket() {
        let args = |t: &Tmux| -> Vec<String> {
            t.command()
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(args(&Tmux::new(Some("s1".into()))), ["-L", "s1"]);
        assert!(args(&Tmux::new(None)).is_empty());
        assert!(args(&Tmux::new(Some(String::new()))).is_empty());
    }

    /// Kills the throwaway server when the test ends, even on failure.
    pub(crate) struct TestServer(pub(crate) Tmux);

    impl TestServer {
        pub(crate) fn start(socket: &str) -> Self {
            let server = TestServer(Tmux::new(Some(socket.into())));
            server.tmux(&["kill-server"]);
            server
        }

        pub(crate) fn tmux(&self, args: &[&str]) {
            let output = self.0.command().args(args).output().unwrap();
            assert!(
                output.status.success() || args == ["kill-server"],
                "tmux {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            self.tmux(&["kill-server"]);
        }
    }

    #[test]
    fn expands_a_leading_tilde_only() {
        let home = Some("/Users/me");
        assert_eq!(expand_tilde("~", home), "/Users/me");
        assert_eq!(expand_tilde("~/work", home), "/Users/me/work");
        assert_eq!(expand_tilde("~bob/x", home), "~bob/x");
        assert_eq!(expand_tilde("/a/~/b", home), "/a/~/b");
        assert_eq!(expand_tilde("~/work", None), "~/work");
    }

    #[test]
    fn creates_renames_and_kills_by_id_with_stable_errors() {
        let server = TestServer::start("tmuxs-test-actions");
        let tmux = &server.0;
        let a = tmux.create_session("alpha", "/tmp").unwrap();
        let b = tmux.create_session("-dash.and:colon", "").unwrap();
        assert!(a.starts_with('$') && a != b);

        let failed = |r: Result<(), TmuxError>| r.unwrap_err().to_string();
        assert!(
            tmux.create_session("alpha", "")
                .unwrap_err()
                .to_string()
                .ends_with("duplicate session: alpha")
        );
        let missing = tmux
            .create_session("x", "/definitely/not/here")
            .unwrap_err();
        assert!(missing.to_string().contains("directory does not exist"));
        assert!(tmux.create_session("f", "/etc/hosts").is_err());

        tmux.rename_session(&b, "-new").unwrap();
        assert!(failed(tmux.rename_session(&a, "-new")).ends_with("duplicate session: -new"));
        assert!(failed(tmux.rename_session("$999", "z")).ends_with("can't find session: $999"));
        tmux.kill_session(&a).unwrap();
        assert!(failed(tmux.kill_session(&a)).ends_with(&format!("can't find session: {a}")));
        let names: Vec<_> = tmux
            .list_sessions()
            .unwrap()
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["-new"]);
    }

    #[test]
    fn no_server_means_no_sessions() {
        let server = TestServer::start("tmuxs-test-empty");
        assert_eq!(server.0.list_sessions(), Ok(vec![]));
    }

    #[test]
    fn lists_real_sessions_sorted_and_distinguishable_by_id() {
        let server = TestServer::start("tmuxs-test");
        for name in ["category", "we.ird:name", "my work", "cat"] {
            server.tmux(&["new-session", "-d", "-s", name]);
        }
        server.tmux(&["new-window", "-t", "=cat:"]);

        let sessions = server.0.list_sessions().unwrap();
        let names: Vec<_> = sessions.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["cat", "category", "my work", "we.ird:name"]);

        let cat = &sessions[0];
        let category = &sessions[1];
        assert_ne!(cat.id, category.id);
        assert_eq!(cat.windows, 2);
        assert_eq!(category.windows, 1);
        assert!(sessions.iter().all(|s| !s.is_attached()));
    }

    #[test]
    fn counts_busy_and_idle_panes_of_a_real_session() {
        let server = TestServer::start("tmuxs-test-panes");
        // One idle pane (the default shell) and one busy pane (`sleep`).
        server.tmux(&["new-session", "-d", "-s", "work", "-n", "main"]);
        server.tmux(&["new-window", "-t", "=work:", "-n", "job", "sleep 300"]);
        server.tmux(&["new-session", "-d", "-s", "other"]);
        let shell = std::env::var("SHELL")
            .ok()
            .and_then(|s| crate::parse::shell_basename(&s));

        // The foreground command can lag behind pane creation, so poll briefly.
        let mut counts = Default::default();
        let mut work = None;
        for _ in 0..50 {
            let sessions = server.0.list_sessions().unwrap();
            let found = sessions.into_iter().find(|s| s.name == "work").unwrap();
            counts = found.pane_counts(shell.as_deref());
            work = Some(found);
            if counts.busy == 1 && counts.idle == 1 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let work = work.unwrap();
        assert_eq!(
            (counts.busy, counts.idle, counts.dead),
            (1, 1, 0),
            "{work:?}"
        );
        assert_eq!(work.panes.len(), 2);
        assert_eq!(crate::parse::window_summary(&work), "0:main 1:job");
        assert!(work.activity > 0);
        assert!(!work.path.is_empty());
    }
}
