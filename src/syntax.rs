use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const KEYWORDS: &[&str] = &[
    "fn",
    "let",
    "mut",
    "pub",
    "impl",
    "struct",
    "enum",
    "trait",
    "if",
    "else",
    "match",
    "for",
    "while",
    "loop",
    "return",
    "use",
    "mod",
    "async",
    "await",
    "const",
    "static",
    "class",
    "def",
    "import",
    "from",
    "export",
    "interface",
    "type",
    "package",
    "func",
    "var",
];

pub fn highlight_content(content: &str, path: &str, max_lines: usize) -> Vec<Line<'static>> {
    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
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

#[cfg(test)]
mod tests {
    use super::highlight_content;
    use ratatui::style::{Color, Modifier, Style};

    #[test]
    fn highlights_rust_keyword_as_cyan_bold() {
        let lines = highlight_content("fn main() {}\n", "main.rs", 10);
        assert_eq!(lines.len(), 1);
        let spans = &lines[0].spans;
        assert!(spans.iter().any(|s| s.content == "fn"
            && s.style == Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    }

    #[test]
    fn highlights_string_as_yellow() {
        let lines = highlight_content("let s = \"hello\";\n", "main.rs", 10);
        assert_eq!(lines.len(), 1);
        let spans = &lines[0].spans;
        assert!(spans.iter().any(|s| s.content == "hello"
            && s.style == Style::default().fg(Color::Yellow)));
    }

    #[test]
    fn highlights_number_as_magenta() {
        let lines = highlight_content("let x = 42;\n", "main.rs", 10);
        let spans = &lines[0].spans;
        assert!(spans.iter().any(|s| s.content == "42"
            && s.style == Style::default().fg(Color::Magenta)));
    }

    #[test]
    fn highlights_comment_as_green() {
        let lines = highlight_content("// this is a comment\nlet x = 1;\n", "main.rs", 10);
        assert_eq!(lines.len(), 2);
        let spans = &lines[0].spans;
        assert!(!spans.is_empty());
        assert_eq!(spans[0].content, "// this is a comment");
        assert_eq!(spans[0].style, Style::default().fg(Color::Green));
    }

    #[test]
    fn python_comment_uses_hash_prefix() {
        let lines = highlight_content("# python comment\n", "main.py", 10);
        assert_eq!(lines.len(), 1);
        let spans = &lines[0].spans;
        assert_eq!(spans[0].style, Style::default().fg(Color::Green));
    }

    #[test]
    fn respects_max_lines() {
        let input = "a\nb\nc\nd\ne\n";
        assert_eq!(highlight_content(input, "f.txt", 3).len(), 3);
        assert_eq!(highlight_content(input, "f.txt", 10).len(), 5);
    }

    #[test]
    fn keywords_highlighted_regardless_of_extension() {
        let lines = highlight_content("fn main()", "Makefile", 10);
        let spans = &lines[0].spans;
        let keyword_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        assert!(spans.iter().any(|s| s.content == "fn" && s.style == keyword_style));
    }

    #[test]
    fn handles_empty_content() {
        let lines = highlight_content("", "main.rs", 10);
        assert!(lines.is_empty());
    }

    #[test]
    fn highlights_comment_prefix_after_whitespace() {
        let lines = highlight_content("  // indented comment\n", "main.rs", 10);
        let spans = &lines[0].spans;
        assert!(!spans.is_empty());
        let comment_span = spans.iter().find(|s| s.content.contains("//")).unwrap();
        assert_eq!(comment_span.style, Style::default().fg(Color::Green));
    }
}
