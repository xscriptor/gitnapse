use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::config_file;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub preferred_clone_dir: String,
    pub last_branch_by_repo: HashMap<String, String>,
}

impl AccountConfig {
    /// Loads the account configuration from disk, or returns a default configuration
    /// if no config file exists yet.
    ///
    /// The default configuration uses the current working directory as the preferred
    /// clone directory and an empty branch history.
    ///
    /// # Errors
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load_or_default() -> Result<Self> {
        let file = config_file()?;
        if !file.exists() {
            let clone_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
                .to_string();
            return Ok(Self {
                preferred_clone_dir: clone_dir,
                last_branch_by_repo: HashMap::new(),
            });
        }

        let raw = fs::read_to_string(&file)
            .with_context(|| format!("Cannot read config file: {}", file.display()))?;
        let cfg: AccountConfig =
            serde_json::from_str(&raw).context("Invalid account config format")?;
        Ok(cfg)
    }

    /// Saves the account configuration to disk as JSON.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn save(&self) -> Result<()> {
        let file = config_file()?;
        let content =
            serde_json::to_string_pretty(self).context("Cannot serialize account config")?;
        fs::write(&file, content)
            .with_context(|| format!("Cannot write config file: {}", file.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AccountConfig;
    use std::collections::HashMap;

    #[test]
    fn roundtrip_serialization() {
        let _dir = tempfile::tempdir().expect("tempdir");

        let mut config = AccountConfig {
            preferred_clone_dir: "/home/user/projects".to_string(),
            last_branch_by_repo: HashMap::new(),
        };
        config
            .last_branch_by_repo
            .insert("owner/repo".to_string(), "main".to_string());
        config
            .last_branch_by_repo
            .insert("owner/other".to_string(), "develop".to_string());

        let json = serde_json::to_string_pretty(&config).expect("serialize");
        let deserialized: AccountConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.preferred_clone_dir, config.preferred_clone_dir);
        assert_eq!(deserialized.last_branch_by_repo.len(), 2);
        assert_eq!(
            deserialized.last_branch_by_repo.get("owner/repo"),
            Some(&"main".to_string())
        );
        assert_eq!(
            deserialized.last_branch_by_repo.get("owner/other"),
            Some(&"develop".to_string())
        );
    }

    #[test]
    fn handles_invalid_json_gracefully() {
        let err = serde_json::from_str::<AccountConfig>("not valid json");
        assert!(err.is_err());
    }

    #[test]
    fn handles_missing_fields() {
        let err = serde_json::from_str::<AccountConfig>(r#"{}"#);
        assert!(err.is_err());
    }
}
