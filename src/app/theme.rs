use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const PALETTE: [(u8, u8, u8); 16] = [
    (0x36, 0x35, 0x37),
    (0xfc, 0x61, 0x8d),
    (0x7b, 0xd8, 0x8f),
    (0xfc, 0xe5, 0x66),
    (0xfd, 0x93, 0x53),
    (0x94, 0x8a, 0xe3),
    (0x5a, 0xd4, 0xe6),
    (0xf7, 0xf1, 0xff),
    (0x69, 0x67, 0x6c),
    (0xfc, 0x61, 0x8d),
    (0x7b, 0xd8, 0x8f),
    (0xfc, 0xe5, 0x66),
    (0xfd, 0x93, 0x53),
    (0x94, 0x8a, 0xe3),
    (0x5a, 0xd4, 0xe6),
    (0xf7, 0xf1, 0xff),
];

#[cfg(test)]
pub fn palette_len() -> usize {
    PALETTE.len()
}

fn palette_rgb(index: usize) -> (u8, u8, u8) {
    PALETTE[index % PALETTE.len()]
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

fn nav_labels() -> [&'static str; 17] {
    [
        " / Search ",
        " Enter Open/Preview ",
        " ↑/↓ Move ",
        " ← Prev Page ",
        " → Next Page ",
        " Tab Repos/Tree/Preview ",
        " PgUp/PgDn Preview ",
        " Home/End Preview ",
        " b Branch ",
        " f Find File ",
        " v Tree View ",
        " d Download File ",
        " c Clone ",
        " t Token ",
        " o OAuth Login ",
        " Esc Back ",
        " Mouse Click/Scroll ",
    ]
}

pub fn nav_hint_lines(max_width: usize) -> Vec<Line<'static>> {
    let width = max_width.max(20);
    let labels = nav_labels();
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
        current.push(Span::styled((*label).to_string(), selection_style(index)));
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
    use super::{contrast_fg_from_rgb, palette_len};
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
}
