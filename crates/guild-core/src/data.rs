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

impl Progress {
    fn path() -> PathBuf {
        GuildConfig::guild_dir().join("data").join("progress.toml")
    }

    /// Load the progress registry from `~/.guild/data/progress.toml`.
    /// Returns an empty progress structure if the file does not exist.
    pub fn load() -> Result<Self, GuildError> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self {
                checkpoints: Vec::new(),
            });
        }
        let content =
            fs::read_to_string(&path).map_err(|_| GuildError::DataError { path: path.clone() })?;
        let progress: Self = toml::from_str(&content)?;
        Ok(progress)
    }

    /// Save the progress to `~/.guild/data/progress.toml`.
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

    /// Mark a checkpoint as completed (case-insensitive).
    /// Returns an error if the checkpoint ID is not found.
    pub fn complete_checkpoint(&mut self, id: &str) -> Result<(), GuildError> {
        let id_lower = id.trim().to_lowercase();
        if let Some(checkpoint) = self
            .checkpoints
            .iter_mut()
            .find(|c| c.id.trim().to_lowercase() == id_lower)
        {
            checkpoint.completed = true;
            Ok(())
        } else {
            Err(GuildError::CheckpointNotFound(id.to_string()))
        }
    }

    /// Return the first incomplete checkpoint in the curriculum sequence.
    pub fn next_incomplete(&self) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| !c.completed)
    }
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
    use tempfile::tempdir;

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

    fn sample_checkpoint(id: &str, title: &str, completed: bool) -> Checkpoint {
        Checkpoint {
            id: id.to_string(),
            title: title.to_string(),
            completed,
        }
    }

    #[test]
    fn test_load_missing_returns_empty_progress() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let progress = Progress::load().unwrap();
        assert!(progress.checkpoints.is_empty());

        restore_home(original);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let mut progress = Progress::load().unwrap();
        progress
            .checkpoints
            .push(sample_checkpoint("id-1", "Title 1", false));
        progress
            .checkpoints
            .push(sample_checkpoint("id-2", "Title 2", true));
        progress.save().unwrap();

        let loaded = Progress::load().unwrap();
        assert_eq!(loaded.checkpoints.len(), 2);
        assert_eq!(loaded.checkpoints[0].id, "id-1");
        assert_eq!(loaded.checkpoints[0].title, "Title 1");
        assert!(!loaded.checkpoints[0].completed);
        assert!(loaded.checkpoints[1].completed);

        restore_home(original);
    }

    #[test]
    fn test_complete_checkpoint_success_case_insensitive() {
        let mut progress = Progress {
            checkpoints: vec![
                sample_checkpoint("checkpoint-1", "Title 1", false),
                sample_checkpoint("checkpoint-2", "Title 2", false),
            ],
        };

        // Exact match
        progress.complete_checkpoint("checkpoint-1").unwrap();
        assert!(progress.checkpoints[0].completed);

        // Case-insensitive & trimmed match
        progress.complete_checkpoint("  CHECKPOINT-2  ").unwrap();
        assert!(progress.checkpoints[1].completed);
    }

    #[test]
    fn test_complete_checkpoint_not_found() {
        let mut progress = Progress {
            checkpoints: vec![sample_checkpoint("checkpoint-1", "Title 1", false)],
        };

        let res = progress.complete_checkpoint("nonexistent");
        assert!(res.is_err());
        match res.unwrap_err() {
            GuildError::CheckpointNotFound(id) => assert_eq!(id, "nonexistent"),
            other => panic!("expected CheckpointNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_next_incomplete() {
        let mut progress = Progress {
            checkpoints: vec![
                sample_checkpoint("checkpoint-1", "Title 1", true),
                sample_checkpoint("checkpoint-2", "Title 2", false),
                sample_checkpoint("checkpoint-3", "Title 3", false),
            ],
        };

        // First incomplete is checkpoint-2
        let next = progress.next_incomplete().unwrap();
        assert_eq!(next.id, "checkpoint-2");

        // Complete checkpoint-2
        progress.complete_checkpoint("checkpoint-2").unwrap();

        // First incomplete is now checkpoint-3
        let next = progress.next_incomplete().unwrap();
        assert_eq!(next.id, "checkpoint-3");

        // Complete checkpoint-3
        progress.complete_checkpoint("checkpoint-3").unwrap();

        // No more incomplete checkpoints
        assert!(progress.next_incomplete().is_none());
    }
}
