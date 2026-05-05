use super::StorageError;
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::{Path, PathBuf};

pub fn named_file(dir: impl Into<PathBuf>, name: &str) -> PathBuf {
    dir.into().join(name)
}

pub fn read_json<T>(path: impl AsRef<Path>) -> Option<T>
where
    T: DeserializeOwned,
{
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn read_json_or_default<T>(path: impl AsRef<Path>) -> T
where
    T: DeserializeOwned + Default,
{
    read_json(path).unwrap_or_default()
}

pub fn write_json_pretty<T>(path: impl AsRef<Path>, value: &T) -> Result<(), StorageError>
where
    T: Serialize,
{
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

pub fn clear_json_object(path: impl AsRef<Path>) -> Result<(), StorageError> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, b"{}")?;
    Ok(())
}
