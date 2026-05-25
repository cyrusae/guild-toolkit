use crate::{GuildError, config::GuildConfig};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Apprentice profile — stored in `~/.guild/data/profile.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub handle: String,
    pub joined: String,
}

impl Profile {
    /// Returns the canonical path for the profile file.
    fn path() -> PathBuf {
        GuildConfig::guild_dir().join("data").join("profile.toml")
    }

    /// Load the profile from `~/.guild/data/profile.toml`.
    pub fn load() -> Result<Self, GuildError> {
        let path = Self::path();
        let content =
            fs::read_to_string(&path).map_err(|_| GuildError::DataError { path: path.clone() })?;
        let profile: Self = toml::from_str(&content)?;
        Ok(profile)
    }

    /// Save the profile to `~/.guild/data/profile.toml`.
    pub fn save(&self) -> Result<(), GuildError> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string(self).map_err(|e| GuildError::SerializeError(e.to_string()))?;
        fs::write(&path, content)?;
        Ok(())
    }
}

/// Project registry entry — stored in `~/.guild/data/projects.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegistry {
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub status: ProjectStatus,
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    NotStarted,
    InProgress,
    UnderReview,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// Curriculum progress — stored in `~/.guild/data/progress.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

/// Review history — stored in `~/.guild/data/reviews.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewHistory {
    pub reviews: Vec<ReviewRound>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRound {
    pub project: String,
    pub round: u32,
    pub status: ReviewStatus,
    pub feedback_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Submitted,
    InReview,
    Returned,
    Accepted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Serialize all tests that mutate $HOME to avoid parallel races.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn mock_home(dir: &std::path::Path) {
        unsafe {
            std::env::set_var("HOME", dir);
        }
    }

    fn restore_home(original: Option<String>) {
        if let Some(home) = original {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    fn sample_profile() -> Profile {
        Profile {
            name: "Alice".to_string(),
            handle: "alice".to_string(),
            joined: "2025-01-01".to_string(),
        }
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let profile = sample_profile();
        profile.save().unwrap();

        let loaded = Profile::load().unwrap();
        assert_eq!(loaded.name, "Alice");
        assert_eq!(loaded.handle, "alice");
        assert_eq!(loaded.joined, "2025-01-01");

        restore_home(original);
    }

    #[test]
    fn test_load_missing_file_returns_data_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());
        // Do NOT create the data directory or file

        let result = Profile::load();
        assert!(result.is_err());
        match result.unwrap_err() {
            GuildError::DataError { .. } => {}
            other => panic!("expected DataError, got: {:?}", other),
        }

        restore_home(original);
    }

    #[test]
    fn test_load_invalid_toml_returns_parse_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let data_dir = dir.path().join(".guild").join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let path = data_dir.join("profile.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "not valid toml !!! @@@").unwrap();

        let result = Profile::load();
        assert!(result.is_err());
        match result.unwrap_err() {
            GuildError::ConfigParse(_) => {}
            other => panic!("expected ConfigParse, got: {:?}", other),
        }

        restore_home(original);
    }

    #[test]
    fn test_load_wrong_schema_returns_parse_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let data_dir = dir.path().join(".guild").join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let path = data_dir.join("profile.toml");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "some_unrelated_key = \"hello\"").unwrap();

        let result = Profile::load();
        assert!(result.is_err());
        match result.unwrap_err() {
            GuildError::ConfigParse(_) => {}
            other => panic!("expected ConfigParse, got: {:?}", other),
        }

        restore_home(original);
    }

    #[test]
    fn test_save_creates_data_directory() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());
        // Deliberately do NOT pre-create ~/.guild/data/

        let profile = sample_profile();
        profile.save().unwrap();

        let expected_path = dir.path().join(".guild").join("data").join("profile.toml");
        assert!(
            expected_path.exists(),
            "profile.toml should have been created"
        );

        restore_home(original);
    }
}
