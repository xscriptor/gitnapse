use serde::Deserialize;

use crate::config::{config_dir, strip_jsonc_comments};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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
