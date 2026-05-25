use crate::GuildError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level guild configuration, loaded from `~/.guild/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub user: UserConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub handle: String,
}

impl GuildConfig {
    /// Returns the default guild directory: `~/.guild/`
    pub fn guild_dir() -> PathBuf {
        dirs_or_home().join(".guild")
    }

    /// Returns the default config file path: `~/.guild/config.toml`
    pub fn default_path() -> PathBuf {
        Self::guild_dir().join("config.toml")
    }

    /// Load config from the default path.
    pub fn load() -> Result<Self, GuildError> {
        Self::load_from(&Self::default_path())
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Result<Self, GuildError> {
        let content = std::fs::read_to_string(path).map_err(|_| GuildError::ConfigNotFound {
            path: path.to_path_buf(),
        })?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_from_valid_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "[user]\nname = \"Alice\"\nhandle = \"alice\"").unwrap();

        let config = GuildConfig::load_from(&path).unwrap();
        assert_eq!(config.user.name, "Alice");
        assert_eq!(config.user.handle, "alice");
    }

    #[test]
    fn test_load_from_missing_file_returns_config_not_found() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");

        let result = GuildConfig::load_from(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            GuildError::ConfigNotFound { path: err_path } => {
                assert_eq!(err_path, path);
            }
            other => panic!("expected ConfigNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_load_from_invalid_toml_returns_parse_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "not valid toml formatting").unwrap();

        let result = GuildConfig::load_from(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            GuildError::ConfigParse(_) => {}
            other => panic!("expected ConfigParse, got: {:?}", other),
        }
    }

    #[test]
    fn test_load_from_valid_toml_but_incorrect_schema_returns_parse_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("wrong_schema.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "some_unrelated_setting = 42").unwrap();

        let result = GuildConfig::load_from(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            GuildError::ConfigParse(_) => {}
            other => panic!("expected ConfigParse, got: {:?}", other),
        }
    }

    #[test]
    fn test_guild_dir_resolves_with_home_env() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let original_home = std::env::var("HOME").ok();

        unsafe {
            std::env::set_var("HOME", "/tmp/mock_home");
        }
        assert_eq!(
            GuildConfig::guild_dir(),
            PathBuf::from("/tmp/mock_home/.guild")
        );
        assert_eq!(
            GuildConfig::default_path(),
            PathBuf::from("/tmp/mock_home/.guild/config.toml")
        );

        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_load_from_default_path() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let original_home = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        let mock_home = dir.path();

        unsafe {
            std::env::set_var("HOME", mock_home);
        }

        let guild_dir = mock_home.join(".guild");
        std::fs::create_dir_all(&guild_dir).unwrap();
        let path = guild_dir.join("config.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "[user]\nname = \"Bob\"\nhandle = \"bob\"").unwrap();

        let config = GuildConfig::load().unwrap();
        assert_eq!(config.user.name, "Bob");
        assert_eq!(config.user.handle, "bob");

        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }
}
