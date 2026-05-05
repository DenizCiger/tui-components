use super::StorageError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct SecretStorageDiagnostic {
    pub available: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SecretStore {
    pub service: String,
    pub label: String,
    pub env_prefix: String,
    pub config_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SecretFileData {
    #[serde(default)]
    entries: HashMap<String, String>,
}

impl SecretStore {
    pub fn new(
        service: impl Into<String>,
        label: impl Into<String>,
        env_prefix: impl Into<String>,
        config_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            service: service.into(),
            label: label.into(),
            env_prefix: env_prefix.into(),
            config_dir: config_dir.into(),
        }
    }

    pub fn diagnostic(&self) -> SecretStorageDiagnostic {
        get_secure_storage_diagnostic()
    }

    pub fn save(&self, account_key: &str, password: &str) -> Result<(), StorageError> {
        if cfg!(target_os = "macos") {
            run_command(
                "security",
                &[
                    "add-generic-password",
                    "-a",
                    account_key,
                    "-s",
                    &self.service,
                    "-w",
                    password,
                    "-U",
                ],
                None,
            )?;
            return Ok(());
        }

        if cfg!(target_os = "linux") {
            run_command(
                "secret-tool",
                &[
                    "store",
                    "--label",
                    &self.label,
                    "service",
                    &self.service,
                    "account",
                    account_key,
                ],
                Some(password),
            )?;
            return Ok(());
        }

        if cfg!(windows) {
            let encrypted = self.encrypt_dpapi(password)?;
            let mut store = self.read_windows_secret_file();
            store.entries.insert(account_key.to_owned(), encrypted);
            self.write_windows_secret_file(&store)?;
            return Ok(());
        }

        Err(StorageError::Message(format!(
            "Unsupported platform '{}' for secure password storage",
            std::env::consts::OS
        )))
    }

    pub fn load(&self, account_key: &str) -> Result<Option<String>, StorageError> {
        if cfg!(target_os = "macos") {
            return run_command(
                "security",
                &["find-generic-password", "-a", account_key, "-s", &self.service, "-w"],
                None,
            )
            .map(Some)
            .or(Ok(None));
        }
        if cfg!(target_os = "linux") {
            return run_command(
                "secret-tool",
                &["lookup", "service", &self.service, "account", account_key],
                None,
            )
            .map(Some)
            .or(Ok(None));
        }
        if cfg!(windows) {
            let store = self.read_windows_secret_file();
            let encrypted = match store.entries.get(account_key) {
                Some(value) => value.clone(),
                None => return Ok(None),
            };
            return self.decrypt_dpapi(&encrypted).map(Some).or(Ok(None));
        }
        Ok(None)
    }

    pub fn clear(&self, account_key: &str) -> Result<(), StorageError> {
        if cfg!(target_os = "macos") {
            let _ = run_command(
                "security",
                &["delete-generic-password", "-a", account_key, "-s", &self.service],
                None,
            );
            return Ok(());
        }
        if cfg!(target_os = "linux") {
            let _ = run_command(
                "secret-tool",
                &["clear", "service", &self.service, "account", account_key],
                None,
            );
            return Ok(());
        }
        if cfg!(windows) {
            let mut store = self.read_windows_secret_file();
            store.entries.remove(account_key);
            self.write_windows_secret_file(&store)?;
        }
        Ok(())
    }

    fn secret_file(&self) -> PathBuf {
        self.config_dir.join("secrets.json")
    }

    fn read_windows_secret_file(&self) -> SecretFileData {
        match fs::read_to_string(self.secret_file()) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => SecretFileData::default(),
        }
    }

    fn write_windows_secret_file(&self, data: &SecretFileData) -> Result<(), StorageError> {
        fs::create_dir_all(&self.config_dir)?;
        fs::write(self.secret_file(), serde_json::to_vec_pretty(data)?)?;
        Ok(())
    }

    fn env_name(&self, suffix: &str) -> String {
        format!("{}_{}", self.env_prefix, suffix)
    }

    fn encrypt_dpapi(&self, plaintext: &str) -> Result<String, StorageError> {
        let env_name = self.env_name("SECRET");
        let script = format!(
            "Add-Type -AssemblyName System.Security;$bytes=[System.Text.Encoding]::UTF8.GetBytes($env:{env_name});$enc=[System.Security.Cryptography.ProtectedData]::Protect($bytes,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser);[Convert]::ToBase64String($enc)"
        );
        run_powershell(&script, &[(&env_name, plaintext)])
    }

    fn decrypt_dpapi(&self, ciphertext_b64: &str) -> Result<String, StorageError> {
        let env_name = self.env_name("SECRET_B64");
        let script = format!(
            "Add-Type -AssemblyName System.Security;$bytes=[Convert]::FromBase64String($env:{env_name});$dec=[System.Security.Cryptography.ProtectedData]::Unprotect($bytes,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser);[System.Text.Encoding]::UTF8.GetString($dec)"
        );
        run_powershell(&script, &[(&env_name, ciphertext_b64)])
    }
}

pub fn get_secure_storage_diagnostic() -> SecretStorageDiagnostic {
    if cfg!(target_os = "macos") {
        if !command_exists("security") {
            return SecretStorageDiagnostic { available: false, message: "macOS Keychain CLI not found; auto-login password storage is unavailable.".to_owned() };
        }
        return SecretStorageDiagnostic { available: true, message: String::new() };
    }
    if cfg!(target_os = "linux") {
        if !command_exists("secret-tool") {
            return SecretStorageDiagnostic { available: false, message: "Install 'secret-tool' (libsecret) to enable secure password storage and auto-login.".to_owned() };
        }
        return SecretStorageDiagnostic { available: true, message: String::new() };
    }
    if cfg!(windows) {
        if windows_shell_command().is_err() {
            return SecretStorageDiagnostic { available: false, message: "PowerShell (powershell.exe or pwsh) is required for secure password storage and auto-login.".to_owned() };
        }
        if run_powershell("Add-Type -AssemblyName System.Security; 'ok'", &[]).is_err() {
            return SecretStorageDiagnostic { available: false, message: "Windows secure storage initialization failed (System.Security unavailable in PowerShell).".to_owned() };
        }
        return SecretStorageDiagnostic { available: true, message: String::new() };
    }
    SecretStorageDiagnostic {
        available: false,
        message: format!("Secure password storage is not supported on platform '{}'.", std::env::consts::OS),
    }
}

fn command_exists(command: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "which" };
    Command::new(checker)
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn windows_shell_command() -> Result<String, StorageError> {
    for candidate in ["powershell.exe", "powershell", "pwsh.exe", "pwsh"] {
        if command_exists(candidate) {
            return Ok(candidate.to_owned());
        }
    }
    Err(StorageError::Message("No PowerShell executable found".to_owned()))
}

fn run_command(command: &str, args: &[&str], input: Option<&str>) -> Result<String, StorageError> {
    let mut process = Command::new(command);
    process.args(args);
    if input.is_some() {
        process.stdin(Stdio::piped());
    }
    process.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = process.spawn()?;
    if let Some(input) = input {
        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(input.as_bytes())?;
        }
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(StorageError::Message(String::from_utf8_lossy(&output.stderr).trim().to_owned()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn run_powershell(script: &str, envs: &[(&str, &str)]) -> Result<String, StorageError> {
    let shell = windows_shell_command()?;
    let mut command = Command::new(shell);
    command.arg("-NoProfile").arg("-NonInteractive").arg("-Command").arg(script);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(StorageError::Message(String::from_utf8_lossy(&output.stderr).trim().to_owned()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[allow(dead_code)]
fn _assert_path(_: &Path) {}
