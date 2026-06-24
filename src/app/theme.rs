use crate::config::{KeybindingsConfig, ThemeConfig, config_dir, strip_jsonc_comments};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::sync::{LazyLock, RwLock};

/// Embedded theme data: (name, JSONC content).
/// Bundled at compile time so themes are available after `cargo install`.
const EMBEDDED_THEMES: &[(&str, &str)] = &[
    ("Berlin", include_str!("../../themes/Berlin.jsonc")),
    ("Bogota", include_str!("../../themes/Bogota.jsonc")),
    ("Helsinki", include_str!("../../themes/Helsinki.jsonc")),
    ("Lahabana", include_str!("../../themes/Lahabana.jsonc")),
    ("London", include_str!("../../themes/London.jsonc")),
    ("Madrid", include_str!("../../themes/Madrid.jsonc")),
    ("Miami", include_str!("../../themes/Miami.jsonc")),
    ("Oslo", include_str!("../../themes/Oslo.jsonc")),
    ("Paris", include_str!("../../themes/Paris.jsonc")),
    ("Praha", include_str!("../../themes/Praha.jsonc")),
    ("Tokio", include_str!("../../themes/Tokio.jsonc")),
    ("X", include_str!("../../themes/X.jsonc")),
];

const DEFAULT_PALETTE: [[u8; 3]; 16] = [
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
];

static PALETTE: LazyLock<RwLock<Vec<[u8; 3]>>> =
    LazyLock::new(|| RwLock::new(DEFAULT_PALETTE.to_vec()));

pub fn init_theme(config: &ThemeConfig) {
    if let Ok(mut p) = PALETTE.write() {
        *p = config.palette.clone();
    }
}

pub fn load_theme_by_name(name: &str) -> ThemeConfig {
    // Try external themes directory first (allows user overrides).
    if let Ok(dir) = config_dir() {
        let path = dir.join("themes").join(format!("{name}.jsonc"));
        if path.exists()
            && let Ok(raw) = std::fs::read_to_string(&path)
        {
            let cleaned = strip_jsonc_comments(&raw);
            if let Ok(cfg) = serde_json::from_str(&cleaned) {
                return cfg;
            }
        }
    }
    // Fall back to embedded themes.
    for (embedded_name, content) in EMBEDDED_THEMES {
        if *embedded_name == name {
            let cleaned = strip_jsonc_comments(content);
            if let Ok(cfg) = serde_json::from_str(&cleaned) {
                return cfg;
            }
        }
    }
    ThemeConfig::default()
}

fn collect_themes_from(dir: &std::path::Path, names: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonc")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                let s = stem.to_string();
                if !names.contains(&s) {
                    names.push(s);
                }
            }
        }
    }
}

pub fn list_available_themes() -> Vec<String> {
    let mut names = Vec::new();

    // External themes take precedence (user overrides).
    if let Ok(dir) = config_dir() {
        collect_themes_from(&dir.join("themes"), &mut names);
    }

    // Always include embedded themes.
    for (name, _) in EMBEDDED_THEMES {
        let s = name.to_string();
        if !names.contains(&s) {
            names.push(s);
        }
    }

    names.sort();
    names
}

fn palette() -> Vec<[u8; 3]> {
    PALETTE
        .read()
        .ok()
        .map(|g| g.clone())
        .unwrap_or(DEFAULT_PALETTE.to_vec())
}

#[cfg(test)]
pub fn palette_len() -> usize {
    palette().len()
}

fn palette_rgb(index: usize) -> (u8, u8, u8) {
    let pal = palette();
    let entry = pal[index % pal.len()];
    (entry[0], entry[1], entry[2])
}

fn contrast_fg_from_rgb(rgb: (u8, u8, u8)) -> Color {
    let (r, g, b) = rgb;
    let luminance = (0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b)) / 255.0;
    if luminance >= 0.58 {
        Color::Black
    } else {
        Color::White
    }
}

pub fn selection_style(index: usize) -> Style {
    let rgb = palette_rgb(index);
    Style::default()
        .bg(Color::Rgb(rgb.0, rgb.1, rgb.2))
        .fg(contrast_fg_from_rgb(rgb))
        .add_modifier(Modifier::BOLD)
}

fn nav_labels(kb: &KeybindingsConfig) -> Vec<String> {
    fn key_str(keys: &[String]) -> String {
        keys.join("/")
    }
    vec![
        format!(" {} Search ", key_str(std::slice::from_ref(&kb.search))),
        format!(
            " {} Open/Preview ",
            key_str(std::slice::from_ref(&kb.enter))
        ),
        format!(" {}/{} Move ", kb.scroll_up, kb.scroll_down),
        format!(" {} Prev Page ", key_str(&kb.page_left)),
        format!(" {} Next Page ", key_str(&kb.page_right)),
        format!(" {} Repos/Tree/Preview ", kb.focus_next),
        format!(" {}/{} Preview ", kb.page_up, kb.page_down),
        format!(" {}/{} Preview ", kb.home, kb.end),
        format!(" {} Branch ", kb.branch_picker),
        format!(" {} Find File ", kb.file_search),
        format!(" {} Tree View ", kb.tree_view),
        format!(" {} Download File ", kb.download),
        format!(" {} Clone ", kb.clone),
        format!(" {} Token ", kb.token_input),
        format!(" {} OAuth State ", kb.oauth_status),
        format!(" {} Back ", kb.escape),
        " Mouse Click/Scroll ".to_string(),
    ]
}

pub fn nav_hint_lines(kb: &KeybindingsConfig, max_width: usize) -> Vec<Line<'static>> {
    let width = max_width.max(20);
    let labels = nav_labels(kb);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    for (index, label) in labels.iter().enumerate() {
        let label_w = label.chars().count();
        if current_width + label_w > width && !current.is_empty() {
            lines.push(Line::from(current));
            current = Vec::new();
            current_width = 0;
        }
        current.push(Span::styled(label.to_string(), selection_style(index)));
        current_width += label_w;
    }

    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    if lines.is_empty() {
        lines.push(Line::from(" Navigation "));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn uses_full_reference_palette() {
        assert_eq!(palette_len(), 16);
    }

    #[test]
    fn picks_white_on_dark_and_black_on_light() {
        assert_eq!(contrast_fg_from_rgb((0x36, 0x35, 0x37)), Color::White);
        assert_eq!(contrast_fg_from_rgb((0xf7, 0xf1, 0xff)), Color::Black);
    }

    #[test]
    fn init_theme_changes_palette() {
        let default_rgb = palette_rgb(1);
        // Debug: show embedded Berlin content
        let embedded = EMBEDDED_THEMES.iter().find(|(n, _)| *n == "Berlin");
        assert!(embedded.is_some(), "Berlin not in EMBEDDED_THEMES");
        if let Some((_, content)) = embedded {
            assert!(content.len() > 0, "Berlin embed content is empty");
            eprintln!("Berlin embed content: {content}");
        }
        // Verify load_theme_by_name returns Berlin
        let berlin = load_theme_by_name("Berlin");
        assert_eq!(
            berlin.palette.len(),
            16,
            "Berlin palette should have 16 entries"
        );
        assert_eq!(berlin.palette[0], [0, 0, 0], "first entry should be black");
        // Apply Berlin theme
        init_theme(&berlin);
        let berlin_rgb = palette_rgb(1);
        assert_eq!(
            berlin_rgb,
            (153, 153, 153),
            "expected Berlin gray, got {berlin_rgb:?}"
        );
        // Reset to default
        init_theme(&ThemeConfig::default());
        let reset_rgb = palette_rgb(1);
        assert_eq!(reset_rgb, default_rgb, "should restore default");
    }
}
