use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::config::{config_dir, strip_jsonc_comments};

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
    pub backspace: String,
    pub delete: String,
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
            backspace: "Backspace".into(),
            delete: "Delete".into(),
        }
    }
}

fn str_to_keycode(s: &str) -> Option<KeyCode> {
    Some(match s {
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "Enter" => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Delete" => KeyCode::Delete,
        single if single.len() == 1 => KeyCode::Char(single.chars().next().unwrap()),
        _ => return None,
    })
}

impl KeybindingsConfig {
    pub fn action_keys(&self, action: &str) -> Vec<String> {
        match action {
            "quit" => vec![self.quit.clone()],
            "search" => vec![self.search.clone()],
            "token_input" => vec![self.token_input.clone()],
            "oauth_status" => vec![self.oauth_status.clone()],
            "clone" => vec![self.clone.clone()],
            "branch_picker" => vec![self.branch_picker.clone()],
            "file_search" => vec![self.file_search.clone()],
            "download" => vec![self.download.clone()],
            "tree_view" => vec![self.tree_view.clone()],
            "focus_next" => vec![self.focus_next.clone()],
            "back" => vec![self.back.clone()],
            "page_left" => self.page_left.clone(),
            "page_right" => self.page_right.clone(),
            "scroll_down" => vec![self.scroll_down.clone()],
            "scroll_up" => vec![self.scroll_up.clone()],
            "page_down" => vec![self.page_down.clone()],
            "page_up" => vec![self.page_up.clone()],
            "home" => vec![self.home.clone()],
            "end" => vec![self.end.clone()],
            "enter" => vec![self.enter.clone()],
            "escape" => vec![self.escape.clone()],
            "backspace" => vec![self.backspace.clone()],
            "delete" => vec![self.delete.clone()],
            _ => vec![],
        }
    }

    pub fn matches_key(&self, action: &str, code: &KeyCode) -> bool {
        self.action_keys(action).iter().any(|key_str| {
            if let Some(kc) = str_to_keycode(key_str) {
                &kc == code
            } else {
                false
            }
        })
    }

    pub fn load_or_default() -> Self {
        let dir = match config_dir() {
            Ok(d) => d,
            Err(_) => return Self::default(),
        };
        let file = dir.join("keybindings.jsonc");
        if !file.exists() {
            // Auto-create default keybindings file as reference
            let defaults = Self::default();
            if let Ok(json) = serde_json::to_string_pretty(&defaults) {
                // Add JSONC comment header
                let content = format!(
                    "// GitNapse Keybindings\n// Uncomment and change values to customize.\n// Restart GitNapse for changes to take effect.\n{}\n",
                    json
                );
                let _ = std::fs::write(&file, content);
            }
            return defaults;
        }
        let raw = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };
        let cleaned = strip_jsonc_comments(&raw);
        serde_json::from_str(&cleaned).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keybindings_roundtrip() {
        let defaults = KeybindingsConfig::default();
        let json = serde_json::to_string_pretty(&defaults).unwrap();
        let back: KeybindingsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(defaults.quit, back.quit);
        assert_eq!(defaults.enter, back.enter);
        assert_eq!(defaults.backspace, back.backspace);
        assert_eq!(defaults.delete, back.delete);
    }

    #[test]
    fn test_matches_key_works() {
        let kb = KeybindingsConfig::default();
        assert!(kb.matches_key("quit", &KeyCode::Char('q')));
        assert!(!kb.matches_key("quit", &KeyCode::Char('x')));
        assert!(kb.matches_key("enter", &KeyCode::Enter));
        assert!(kb.matches_key("escape", &KeyCode::Esc));
        assert!(kb.matches_key("backspace", &KeyCode::Backspace));
        assert!(kb.matches_key("delete", &KeyCode::Delete));
        assert!(kb.matches_key("scroll_up", &KeyCode::Up));
        assert!(kb.matches_key("scroll_down", &KeyCode::Down));
    }
}
