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
