pub mod json;
pub mod secret;
pub mod session;
pub mod time;

use directories::BaseDirs;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn app_config_dir(
    app_dir_name: &str,
    env_override: Option<&str>,
) -> Result<PathBuf, StorageError> {
    if let Some(env_name) = env_override {
        if let Ok(custom) = std::env::var(env_name) {
            if !custom.trim().is_empty() {
                return Ok(PathBuf::from(custom));
            }
        }
    }

    let base_dirs = BaseDirs::new()
        .ok_or_else(|| StorageError::Message("Failed to determine home directory".to_owned()))?;
    Ok(base_dirs.home_dir().join(".config").join(app_dir_name))
}
