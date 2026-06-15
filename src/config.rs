use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub quit: String,
    pub search: String,
    pub token_input: String,
    pub oauth_status: String,
    pub clone: String,
    pub branch_picker: String,
    pub file_search: String,
    pub download: String,
    pub tree_view: String,
    pub focus_next: String,
    pub back: String,
    pub page_left: Vec<String>,
    pub page_right: Vec<String>,
    pub scroll_down: String,
    pub scroll_up: String,
    pub page_down: String,
    pub page_up: String,
    pub home: String,
    pub end: String,
    pub enter: String,
    pub escape: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit: "q".into(),
            search: "/".into(),
            token_input: "t".into(),
            oauth_status: "o".into(),
            clone: "c".into(),
            branch_picker: "b".into(),
            file_search: "f".into(),
            download: "d".into(),
            tree_view: "v".into(),
            focus_next: "Tab".into(),
            back: "Esc".into(),
            page_left: vec!["Left".into(), "[".into()],
            page_right: vec!["Right".into(), "]".into()],
            scroll_down: "Down".into(),
            scroll_up: "Up".into(),
            page_down: "PageDown".into(),
            page_up: "PageUp".into(),
            home: "Home".into(),
            end: "End".into(),
            enter: "Enter".into(),
            escape: "Esc".into(),
        }
    }
}

impl KeybindingsConfig {
    pub fn load_or_default() -> Self {
        let dir = match config_dir() {
            Ok(d) => d,
            Err(_) => return Self::default(),
        };
        let file = dir.join("keybindings.jsonc");
        if !file.exists() {
            return Self::default();
        }
        let raw = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };
        let cleaned = strip_jsonc_comments(&raw);
        serde_json::from_str(&cleaned).unwrap_or_default()
    }
}

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

/// Returns the path to the account configuration file, creating the config
/// directory if it does not exist.
///
/// # Errors
/// Returns an error if the project config directory cannot be resolved or created.
pub fn config_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
        .ok_or_else(|| anyhow!("Unable to resolve project config directory"))?;
    let dir = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&dir)
        .with_context(|| format!("Cannot create config directory: {}", dir.display()))?;
    Ok(dir)
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("account.json"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub palette: Vec<[u8; 3]>,
    #[serde(default = "default_theme_name")]
    #[allow(dead_code)]
    pub theme_name: String,
}

fn default_theme_name() -> String {
    "X".to_string()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            palette: vec![
                [0x36, 0x35, 0x37],
                [0xfc, 0x61, 0x8d],
                [0x7b, 0xd8, 0x8f],
                [0xfc, 0xe5, 0x66],
                [0xfd, 0x93, 0x53],
                [0x94, 0x8a, 0xe3],
                [0x5a, 0xd4, 0xe6],
                [0xf7, 0xf1, 0xff],
                [0x69, 0x67, 0x6c],
                [0xfc, 0x61, 0x8d],
                [0x7b, 0xd8, 0x8f],
                [0xfc, 0xe5, 0x66],
                [0xfd, 0x93, 0x53],
                [0x94, 0x8a, 0xe3],
                [0x5a, 0xd4, 0xe6],
                [0xf7, 0xf1, 0xff],
            ],
            theme_name: "X".to_string(),
        }
    }
}

impl ThemeConfig {
    pub fn load_or_default() -> Self {
        let dir = match config_dir() {
            Ok(d) => d,
            Err(_) => return Self::default(),
        };

        // Auto-install themes on first run
        let themes_dir = dir.join("themes");
        if !themes_dir.exists() {
            let _ = std::fs::create_dir_all(&themes_dir);
            if let Ok(exe_dir) = std::env::current_exe()
                && let Some(exe_parent) = exe_dir.parent()
            {
                let builtin_themes = exe_parent.join("../themes");
                if builtin_themes.exists()
                    && let Ok(entries) = std::fs::read_dir(&builtin_themes)
                {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|ext| ext == "jsonc")
                            && let Some(name) = path.file_name()
                        {
                            let dest = themes_dir.join(name);
                            let _ = std::fs::copy(&path, &dest);
                        }
                    }
                }
            }
        }

        // Try loading the configured theme
        let theme_name = {
            let theme_file = dir.join("theme.jsonc");
            if theme_file.exists() {
                if let Ok(raw) = std::fs::read_to_string(&theme_file) {
                    let cleaned = strip_jsonc_comments(&raw);
                    if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                        cfg.get("theme_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("X")
                            .to_string()
                    } else {
                        "X".to_string()
                    }
                } else {
                    "X".to_string()
                }
            } else {
                "X".to_string()
            }
        };

        // Load the theme file by name
        if let Some(theme_path) = Self::theme_file_path(&theme_name)
            && theme_path.exists()
            && let Ok(raw) = std::fs::read_to_string(&theme_path)
        {
            let cleaned = strip_jsonc_comments(&raw);
            if let Ok(cfg) = serde_json::from_str::<Self>(&cleaned) {
                return cfg;
            }
        }

        Self::default()
    }

    pub fn theme_file_path(name: &str) -> Option<PathBuf> {
        // Check config dir first
        if let Ok(dir) = config_dir() {
            let path = dir.join("themes").join(format!("{name}.jsonc"));
            if path.exists() {
                return Some(path);
            }
        }

        // Check built-in themes directory
        if let Ok(exe_dir) = std::env::current_exe()
            && let Some(exe_parent) = exe_dir.parent()
        {
            let path = exe_parent.join("../themes").join(format!("{name}.jsonc"));
            if path.exists() {
                return Some(path);
            }
        }

        // Check relative to CARGO_MANIFEST_DIR (for development)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let path = PathBuf::from(manifest_dir)
                .join("themes")
                .join(format!("{name}.jsonc"));
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

pub(crate) fn strip_jsonc_comments(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
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
