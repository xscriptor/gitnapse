use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const KEYRING_SERVICE: &str = "com.GitNapse.GitNapse";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretBackend {
    Keyring,
    File,
}

fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .is_some()
    {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(version) = fs::read_to_string("/proc/version")
            && version.to_ascii_lowercase().contains("microsoft")
        {
            return true;
        }
    }
    false
}

fn should_try_keyring() -> bool {
    !is_wsl()
}

fn keyring_get(secret_key: &str) -> Option<Result<Option<String>>> {
    if !should_try_keyring() {
        return None;
    }
    let entry = keyring::Entry::new(KEYRING_SERVICE, secret_key)
        .map_err(anyhow::Error::from)
        .context("Cannot initialize keyring entry");
    match entry {
        Ok(entry) => match entry.get_password() {
            Ok(value) => Some(Ok(Some(value))),
            Err(_) => Some(Ok(None)),
        },
        Err(error) => Some(Err(error)),
    }
}

fn keyring_set(secret_key: &str, value: &str) -> Option<Result<()>> {
    if !should_try_keyring() {
        return None;
    }
    let entry = keyring::Entry::new(KEYRING_SERVICE, secret_key)
        .map_err(anyhow::Error::from)
        .context("Cannot initialize keyring entry");
    match entry {
        Ok(entry) => Some(
            entry
                .set_password(value)
                .map_err(anyhow::Error::from)
                .context("Cannot write secret to keyring"),
        ),
        Err(error) => Some(Err(error)),
    }
}

fn keyring_delete(secret_key: &str) -> Option<Result<()>> {
    if !should_try_keyring() {
        return None;
    }
    let entry = keyring::Entry::new(KEYRING_SERVICE, secret_key)
        .map_err(anyhow::Error::from)
        .context("Cannot initialize keyring entry");
    match entry {
        Ok(entry) => Some(
            entry
                .delete_credential()
                .map_err(anyhow::Error::from)
                .context("Cannot delete keyring secret"),
        ),
        Err(error) => Some(Err(error)),
    }
}

fn file_read(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let value = fs::read_to_string(path)
        .with_context(|| format!("Cannot read secret file: {}", path.display()))?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed))
}

fn file_write(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create secret directory: {}", parent.display()))?;
    }
    fs::write(path, format!("{value}\n"))
        .with_context(|| format!("Cannot write secret file: {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Cannot set secure permissions on {}", path.display()))?;
    }
    Ok(())
}

fn file_delete(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("Cannot remove secret file: {}", path.display()))?;
    }
    Ok(())
}

pub fn save_secret(secret_key: &str, fallback_file: &Path, value: &str) -> Result<SecretBackend> {
    if let Some(result) = keyring_set(secret_key, value)
        && result.is_ok()
    {
        let _ = file_delete(fallback_file);
        return Ok(SecretBackend::Keyring);
    }
    file_write(fallback_file, value)?;
    Ok(SecretBackend::File)
}

pub fn load_secret(secret_key: &str, fallback_file: &Path) -> Result<Option<String>> {
    if let Some(result) = keyring_get(secret_key)
        && let Ok(Some(value)) = result
    {
        return Ok(Some(value));
    }
    file_read(fallback_file)
}

pub fn clear_secret(secret_key: &str, fallback_file: &Path) -> Result<()> {
    if let Some(result) = keyring_delete(secret_key) {
        let _ = result;
    }
    file_delete(fallback_file)
}

pub fn preferred_backend_name() -> &'static str {
    if should_try_keyring() {
        "keyring"
    } else {
        "file-fallback"
    }
}
