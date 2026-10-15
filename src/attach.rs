//! The Attach command and the nested-tmux check. Pure, so both are unit-testable.

/// Shown when Attach is refused because `tmuxs` runs inside tmux.
pub const NESTED_MESSAGE: &str = "Already inside tmux. Detach first (prefix + d), then run tmuxs.";

/// The program and arguments for `tmux [-L socket] [-CC] attach -t <id>`.
pub fn attach_command(iterm2_mode: bool, socket: Option<&str>, id: &str) -> (String, Vec<String>) {
    let mut args = Vec::new();
    if let Some(socket) = socket {
        args.extend(["-L".to_string(), socket.to_string()]);
    }
    if iterm2_mode {
        args.push("-CC".to_string());
    }
    args.extend(["attach".to_string(), "-t".to_string(), id.to_string()]);
    ("tmux".to_string(), args)
}

/// Whether Attach must be refused, given the value of `$TMUX`.
pub fn is_nested(tmux_env: Option<&str>) -> bool {
    tmux_env.is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(mode: bool, socket: Option<&str>) -> Vec<String> {
        let (program, args) = attach_command(mode, socket, "$3");
        assert_eq!(program, "tmux");
        args
    }

    #[test]
    fn plain_attach() {
        assert_eq!(cmd(false, None), ["attach", "-t", "$3"]);
    }

    #[test]
    fn iterm2_mode_adds_cc_before_attach() {
        assert_eq!(cmd(true, None), ["-CC", "attach", "-t", "$3"]);
    }

    #[test]
    fn socket_comes_first() {
        assert_eq!(cmd(false, Some("s")), ["-L", "s", "attach", "-t", "$3"]);
        assert_eq!(
            cmd(true, Some("s")),
            ["-L", "s", "-CC", "attach", "-t", "$3"]
        );
    }

    #[test]
    fn nested_when_tmux_env_is_set_and_non_empty() {
        assert!(is_nested(Some("/tmp/tmux-1/default,123,0")));
        assert!(!is_nested(Some("")));
        assert!(!is_nested(None));
    }

    /// Kills the throwaway server when the test ends, even on failure.
    struct Server(crate::tmux::Tmux);

    impl Server {
        fn run(&self, args: &[&str]) -> std::process::Output {
            let mut command = self.0.command();
            command.args(args).stdin(std::process::Stdio::null());
            command.output().unwrap()
        }
    }

    impl Drop for Server {
        fn drop(&mut self) {
            self.run(&["kill-server"]);
        }
    }

    /// Runs the attach command for `id` without a terminal and returns stderr.
    fn try_attach(mode: bool, id: &str) -> String {
        let (program, args) = attach_command(mode, Some("tmuxs-test-attach"), id);
        let output = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert!(!output.status.success());
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    /// Without a terminal tmux cannot attach, but it resolves the target first:
    /// "not a terminal" proves the id was accepted, "can't find" means it was not.
    /// Needs no controlling terminal on stdin; run it from `cargo test` as usual.
    #[test]
    fn real_tmux_accepts_an_id_target_in_both_forms() {
        let server = Server(crate::tmux::Tmux::new(Some("tmuxs-test-attach".into())));
        server.run(&["kill-server"]);
        for name in ["cat", "category"] {
            assert!(
                server
                    .run(&["new-session", "-d", "-s", name])
                    .status
                    .success()
            );
        }
        let sessions = server.0.list_sessions().unwrap();
        let category = sessions.iter().find(|s| s.name == "category").unwrap();
        assert!(try_attach(false, &category.id).contains("not a terminal"));
        assert!(try_attach(false, "$999").contains("can't find"));
        // -CC checks the terminal attributes before it looks at the target, so
        // without a terminal only the argument parsing is proven here.
        assert!(try_attach(true, &category.id).contains("tcgetattr failed"));
    }
}
