//! The iTerm2 mode setting: how it is resolved at startup and saved to `settings.toml`.

use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Environment variable that sets the iTerm2 mode (`1` or `0`).
pub const CC_ENV: &str = "TMUXS_CC";
/// Environment variable that replaces the directory holding `settings.toml`.
pub const CONFIG_DIR_ENV: &str = "TMUXS_CONFIG_DIR";

const FILE_NAME: &str = "settings.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
struct File {
    iterm2_mode: Option<bool>,
}

/// Picks the initial iTerm2 mode: flag, then `TMUXS_CC`, then the saved value,
/// then ON only when running in iTerm2.
pub fn resolve_iterm2_mode(
    flag: Option<bool>,
    env_cc: Option<&str>,
    saved: Option<bool>,
    term_program: Option<&str>,
) -> bool {
    let from_env = match env_cc {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    };
    flag.or(from_env)
        .or(saved)
        .unwrap_or(term_program == Some("iTerm.app"))
}

/// Where `settings.toml` lives. No home directory means no persistence.
#[derive(Debug, Default, Clone)]
pub struct Settings {
    path: Option<PathBuf>,
}

impl Settings {
    /// `config_dir` is the value of `TMUXS_CONFIG_DIR`; empty counts as unset.
    pub fn new(config_dir: Option<&str>) -> Self {
        let dir = match config_dir.filter(|d| !d.is_empty()) {
            Some(dir) => Some(PathBuf::from(dir)),
            None => choose_app_strategy(AppStrategyArgs {
                top_level_domain: String::new(),
                author: String::new(),
                app_name: "tmuxs".into(),
            })
            .ok()
            .map(|strategy| strategy.config_dir()),
        };
        Self {
            path: dir.map(|dir| dir.join(FILE_NAME)),
        }
    }

    pub fn from_env() -> Self {
        Self::new(std::env::var(CONFIG_DIR_ENV).ok().as_deref())
    }

    /// The saved iTerm2 mode. A missing, unreadable or corrupt file gives `None`.
    pub fn load_iterm2_mode(&self) -> Option<bool> {
        let text = std::fs::read_to_string(self.path.as_ref()?).ok()?;
        toml::from_str::<File>(&text).ok()?.iterm2_mode
    }

    pub fn save_iterm2_mode(&self, on: bool) -> Result<(), String> {
        let path = self.path.as_ref().ok_or("no settings directory found")?;
        let text = toml::to_string(&File {
            iterm2_mode: Some(on),
        })
        .map_err(|e| e.to_string())?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, text).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITERM: Option<&str> = Some("iTerm.app");

    #[test]
    fn flag_beats_env_beats_saved_beats_auto() {
        assert!(!resolve_iterm2_mode(
            Some(false),
            Some("1"),
            Some(true),
            ITERM
        ));
        assert!(resolve_iterm2_mode(None, Some("1"), Some(false), None));
        assert!(!resolve_iterm2_mode(None, Some("0"), Some(true), ITERM));
        assert!(resolve_iterm2_mode(None, None, Some(true), None));
        assert!(!resolve_iterm2_mode(None, None, Some(false), ITERM));
    }

    #[test]
    fn auto_default_is_on_only_in_iterm2() {
        assert!(resolve_iterm2_mode(None, None, None, ITERM));
        assert!(!resolve_iterm2_mode(
            None,
            None,
            None,
            Some("Apple_Terminal")
        ));
        assert!(!resolve_iterm2_mode(None, None, None, None));
    }

    #[test]
    fn invalid_env_value_is_ignored() {
        assert!(resolve_iterm2_mode(None, Some("yes"), Some(true), None));
        assert!(!resolve_iterm2_mode(None, Some(""), None, None));
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tmuxs-test-settings-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_file_loads_as_none() {
        let dir = temp_dir("missing");
        assert_eq!(Settings::new(dir.to_str()).load_iterm2_mode(), None);
    }

    #[test]
    fn corrupt_or_unreadable_file_loads_as_none() {
        let dir = temp_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        let settings = Settings::new(dir.to_str());
        for content in ["not = [toml", "iterm2_mode = \"yes\"", ""] {
            std::fs::write(dir.join(FILE_NAME), content).unwrap();
            assert_eq!(settings.load_iterm2_mode(), None, "{content:?}");
        }
        // A directory where the file should be is unreadable.
        std::fs::remove_file(dir.join(FILE_NAME)).unwrap();
        std::fs::create_dir(dir.join(FILE_NAME)).unwrap();
        assert_eq!(settings.load_iterm2_mode(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_the_directory_and_round_trips() {
        let root = temp_dir("roundtrip");
        let dir = root.join("nested");
        let settings = Settings::new(dir.to_str());
        settings.save_iterm2_mode(true).unwrap();
        assert_eq!(settings.load_iterm2_mode(), Some(true));
        settings.save_iterm2_mode(false).unwrap();
        assert_eq!(settings.load_iterm2_mode(), Some(false));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn save_failure_is_an_error_not_a_panic() {
        let dir = temp_dir("blocked");
        std::fs::create_dir_all(&dir).unwrap();
        // The config "directory" is a file, so it cannot be created.
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, "").unwrap();
        let settings = Settings::new(blocker.to_str());
        assert!(settings.save_iterm2_mode(true).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_directory_means_no_persistence() {
        let settings = Settings { path: None };
        assert_eq!(settings.load_iterm2_mode(), None);
        assert!(settings.save_iterm2_mode(true).is_err());
    }
}
