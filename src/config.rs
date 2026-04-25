use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub preferred_clone_dir: String,
    pub last_branch_by_repo: HashMap<String, String>,
}

impl AccountConfig {
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

    pub fn save(&self) -> Result<()> {
        let file = config_file()?;
        let content =
            serde_json::to_string_pretty(self).context("Cannot serialize account config")?;
        fs::write(&file, content)
            .with_context(|| format!("Cannot write config file: {}", file.display()))?;
        Ok(())
    }
}

pub fn config_file() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
        .ok_or_else(|| anyhow!("Unable to resolve project config directory"))?;
    fs::create_dir_all(dirs.config_dir()).with_context(|| {
        format!(
            "Cannot create config directory: {}",
            dirs.config_dir().display()
        )
    })?;
    Ok(Path::new(dirs.config_dir()).join("account.json"))
}
