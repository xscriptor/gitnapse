use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "impl", "struct", "enum", "trait", "if", "else", "match", "for",
    "while", "loop", "return", "use", "mod", "async", "await", "const", "static", "class", "def",
    "import", "from", "export", "interface", "type", "package", "func", "var",
];

pub fn highlight_content(content: &str, path: &str, max_lines: usize) -> Vec<Line<'static>> {
    let ext = path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    let comment_prefix = match ext.as_str() {
        "py" | "sh" | "toml" | "yaml" | "yml" | "rb" => "#",
        "rs" | "js" | "ts" | "tsx" | "java" | "c" | "cpp" | "go" | "swift" | "kt" => "//",
        _ => "",
    };

    content
        .lines()
        .take(max_lines)
        .map(|line| highlight_line(line, comment_prefix))
        .collect()
}

fn highlight_line(line: &str, comment_prefix: &str) -> Line<'static> {
    if !comment_prefix.is_empty() && line.trim_start().starts_with(comment_prefix) {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Green),
        ));
    }

    let mut spans = Vec::new();
    let mut current = String::new();
    let mut in_string = false;

    for ch in line.chars() {
        if ch == '"' {
            if !current.is_empty() {
                push_token(&mut spans, &current, in_string);
                current.clear();
            }
            in_string = !in_string;
            spans.push(Span::styled(
                "\"".to_string(),
                Style::default().fg(Color::Yellow),
            ));
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
            continue;
        }

        if !current.is_empty() {
            push_token(&mut spans, &current, in_string);
            current.clear();
        }
        spans.push(Span::raw(ch.to_string()));
    }

    if !current.is_empty() {
        push_token(&mut spans, &current, in_string);
    }

    Line::from(spans)
}

fn push_token(spans: &mut Vec<Span<'static>>, token: &str, in_string: bool) {
    if in_string {
        spans.push(Span::styled(
            token.to_string(),
            Style::default().fg(Color::Yellow),
        ));
        return;
    }

    if KEYWORDS.contains(&token) {
        spans.push(Span::styled(
            token.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        return;
    }

    if token.chars().all(|c| c.is_ascii_digit()) {
        spans.push(Span::styled(
            token.to_string(),
            Style::default().fg(Color::Magenta),
        ));
        return;
    }

    spans.push(Span::raw(token.to_string()));
}
