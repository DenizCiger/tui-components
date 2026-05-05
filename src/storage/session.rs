use super::StorageError;
use super::json::{read_json_or_default, write_json_pretty};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionFlags {
    #[serde(default)]
    pub auto_login: bool,
}

pub fn read_session(path: impl AsRef<Path>) -> SessionFlags {
    read_json_or_default(path)
}

pub fn write_session(path: impl AsRef<Path>, flags: &SessionFlags) -> Result<(), StorageError> {
    write_json_pretty(path, flags)
}

pub fn set_auto_login(path: impl AsRef<Path>, value: bool) -> Result<(), StorageError> {
    let mut flags = read_session(path.as_ref());
    flags.auto_login = value;
    write_session(path, &flags)
}
