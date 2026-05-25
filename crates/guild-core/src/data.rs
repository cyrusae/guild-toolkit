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

impl ProjectRegistry {
    fn path() -> PathBuf {
        GuildConfig::guild_dir().join("data").join("projects.toml")
    }

    /// Load the project registry from `~/.guild/data/projects.toml`.
    /// Returns an empty registry if the file does not exist.
    pub fn load() -> Result<Self, GuildError> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self {
                projects: Vec::new(),
            });
        }
        let content =
            fs::read_to_string(&path).map_err(|_| GuildError::DataError { path: path.clone() })?;
        let registry: Self = toml::from_str(&content)?;
        Ok(registry)
    }

    /// Save the project registry to `~/.guild/data/projects.toml`.
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

    /// Add a project to the registry. Returns an error if a project with the same name already exists (case-insensitive).
    /// Trims names and paths, validating character sets.
    pub fn add_project(&mut self, mut project: Project) -> Result<(), GuildError> {
        let trimmed_name = project.name.trim().to_string();
        if trimmed_name.is_empty() {
            return Err(GuildError::InvalidProjectProperties(
                "project name cannot be empty".to_string(),
            ));
        }

        // Enforce safe characters for name (alphanumeric, spaces, hyphens, underscores, dots)
        if !trimmed_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.')
        {
            return Err(GuildError::InvalidProjectProperties(
                "project name can only contain alphanumeric characters, spaces, hyphens, underscores, and dots".to_string(),
            ));
        }

        let trimmed_path = project.path.trim().to_string();
        if trimmed_path.is_empty() {
            return Err(GuildError::InvalidProjectProperties(
                "project path cannot be empty".to_string(),
            ));
        }

        // Path cannot contain null bytes
        if trimmed_path.contains('\0') {
            return Err(GuildError::InvalidProjectProperties(
                "project path cannot contain null bytes".to_string(),
            ));
        }

        let name_lower = trimmed_name.to_lowercase();
        if self
            .projects
            .iter()
            .any(|p| p.name.trim().to_lowercase() == name_lower)
        {
            return Err(GuildError::DuplicateProject(trimmed_name));
        }

        project.name = trimmed_name;
        project.path = trimmed_path;
        self.projects.push(project);
        Ok(())
    }

    /// Look up a project by name (case-insensitive, trimmed).
    pub fn find_project(&self, name: &str) -> Option<&Project> {
        let name_lower = name.trim().to_lowercase();
        self.projects
            .iter()
            .find(|p| p.name.trim().to_lowercase() == name_lower)
    }
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
    use std::sync::Mutex;
    use tempfile::tempdir;

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

    fn sample_project(name: &str, path: &str) -> Project {
        Project {
            name: name.to_string(),
            path: path.to_string(),
            status: ProjectStatus::NotStarted,
            difficulty: Difficulty::Beginner,
        }
    }

    #[test]
    fn test_load_missing_returns_empty_registry() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let registry = ProjectRegistry::load().unwrap();
        assert!(registry.projects.is_empty());

        restore_home(original);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("HOME").ok();
        let dir = tempdir().unwrap();
        mock_home(dir.path());

        let mut registry = ProjectRegistry::load().unwrap();
        registry
            .add_project(sample_project("Project-A", "./a"))
            .unwrap();
        registry
            .add_project(sample_project("Project-B", "./b"))
            .unwrap();
        registry.save().unwrap();

        let loaded = ProjectRegistry::load().unwrap();
        assert_eq!(loaded.projects.len(), 2);

        let p_a = loaded.find_project("project-a").unwrap();
        assert_eq!(p_a.name, "Project-A");
        assert_eq!(p_a.path, "./a");

        let p_b = loaded.find_project("PROJECT-B").unwrap();
        assert_eq!(p_b.name, "Project-B");
        assert_eq!(p_b.path, "./b");

        restore_home(original);
    }

    #[test]
    fn test_add_project_success_and_trimming() {
        let mut registry = ProjectRegistry { projects: vec![] };

        // Trims name and path
        registry
            .add_project(sample_project("  My-Project.1  ", "  /some/path  "))
            .unwrap();

        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.projects[0].name, "My-Project.1");
        assert_eq!(registry.projects[0].path, "/some/path");
    }

    #[test]
    fn test_add_project_duplicate_fails_case_insensitively() {
        let mut registry = ProjectRegistry { projects: vec![] };
        registry
            .add_project(sample_project("My-Project", "/path1"))
            .unwrap();

        // Exact duplicate
        let res1 = registry.add_project(sample_project("My-Project", "/path2"));
        assert!(res1.is_err());
        match res1.unwrap_err() {
            GuildError::DuplicateProject(name) => assert_eq!(name, "My-Project"),
            other => panic!("expected DuplicateProject, got: {:?}", other),
        }

        // Case-insensitive duplicate
        let res2 = registry.add_project(sample_project("my-project", "/path3"));
        assert!(res2.is_err());
        match res2.unwrap_err() {
            GuildError::DuplicateProject(name) => assert_eq!(name, "my-project"),
            other => panic!("expected DuplicateProject, got: {:?}", other),
        }
    }

    #[test]
    fn test_add_project_invalid_properties() {
        let mut registry = ProjectRegistry { projects: vec![] };

        // Empty name
        let res_empty_name = registry.add_project(sample_project("   ", "/path"));
        assert!(res_empty_name.is_err());
        match res_empty_name.unwrap_err() {
            GuildError::InvalidProjectProperties(msg) => assert!(msg.contains("name")),
            other => panic!("expected InvalidProjectProperties, got: {:?}", other),
        }

        // Empty path
        let res_empty_path = registry.add_project(sample_project("Project-A", "   "));
        assert!(res_empty_path.is_err());
        match res_empty_path.unwrap_err() {
            GuildError::InvalidProjectProperties(msg) => assert!(msg.contains("path")),
            other => panic!("expected InvalidProjectProperties, got: {:?}", other),
        }

        // Invalid characters in name (slash /)
        let res_invalid_char = registry.add_project(sample_project("Project/A", "/path"));
        assert!(res_invalid_char.is_err());
        match res_invalid_char.unwrap_err() {
            GuildError::InvalidProjectProperties(msg) => assert!(msg.contains("alphanumeric")),
            other => panic!("expected InvalidProjectProperties, got: {:?}", other),
        }

        // Path containing null bytes
        let res_null_bytes =
            registry.add_project(sample_project("Project-A", "/path\0with\0nulls"));
        assert!(res_null_bytes.is_err());
        match res_null_bytes.unwrap_err() {
            GuildError::InvalidProjectProperties(msg) => assert!(msg.contains("null")),
            other => panic!("expected InvalidProjectProperties, got: {:?}", other),
        }
    }

    #[test]
    fn test_find_project() {
        let mut registry = ProjectRegistry { projects: vec![] };
        registry
            .add_project(sample_project("Project-A", "/path-a"))
            .unwrap();

        // Exact match
        assert!(registry.find_project("Project-A").is_some());
        // Case insensitive match
        assert!(registry.find_project("project-a").is_some());
        // Trimmed search
        assert!(registry.find_project("  project-a  ").is_some());
        // Non-existent search
        assert!(registry.find_project("Project-B").is_none());
    }
}
