pub mod config;
pub mod crosslink;
pub mod data;
pub mod error;
pub mod output;

pub use config::GuildConfig;
pub use error::GuildError;

#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
