use std::path::PathBuf;

/// Shared error type for guild tools.
#[derive(Debug, thiserror::Error)]
pub enum GuildError {
    #[error("config not found: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("failed to parse config: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("data file corrupt or missing: {path}")]
    DataError { path: PathBuf },

    #[error("crosslink command failed: {0}")]
    Crosslink(String),

    #[error("failed to serialize data: {0}")]
    SerializeError(String),

    #[error("review round {1} for project '{0}' already exists")]
    DuplicateReviewRound(String, u32),

    #[error("invalid review round: {0}")]
    InvalidReviewRound(String),
}
