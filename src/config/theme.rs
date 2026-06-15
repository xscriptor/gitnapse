use serde::Deserialize;
use std::path::PathBuf;

use crate::config::{config_dir, strip_jsonc_comments};

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
