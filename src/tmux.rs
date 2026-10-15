//! Runs tmux through `std::process::Command`.

use crate::parse::{SESSION_FORMAT, Session, parse_sessions, sort_sessions};
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
        Ok(sessions)
    }
}

fn is_no_server(stderr: &str) -> bool {
    stderr.contains("no server running") || stderr.contains("error connecting to")
}

#[cfg(test)]
mod tests {
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
    struct TestServer(Tmux);

    impl TestServer {
        fn start(socket: &str) -> Self {
            let server = TestServer(Tmux::new(Some(socket.into())));
            server.tmux(&["kill-server"]);
            server
        }

        fn tmux(&self, args: &[&str]) {
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
}
