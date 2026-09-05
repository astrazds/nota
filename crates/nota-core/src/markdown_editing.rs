//! Markdown formatting over validated UTF-8 byte ranges.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownCommand {
    Bold,
    Italic,
    Strikethrough,
    TaskList,
    Table,
}

impl MarkdownCommand {
    fn affixes(self) -> (&'static str, &'static str) {
        match self {
            Self::Bold => ("**", "**"),
            Self::Italic => ("*", "*"),
            Self::Strikethrough => ("~~", "~~"),
            Self::TaskList => ("- [ ] ", ""),
            Self::Table => (
                "\n| Column 1 | Column 2 |\n| --- | --- |\n| Value 1 | Value 2 |\n",
                "",
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSelection {
    start: usize,
    end: usize,
}

impl ByteSelection {
    pub fn new(content: &str, start: usize, end: usize) -> Option<Self> {
        if start <= content.len()
            && end <= content.len()
            && content.is_char_boundary(start)
            && content.is_char_boundary(end)
        {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub fn ordered(self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattingResult {
    pub content: String,
    pub caret_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownCheatsheetSection {
    pub title: &'static str,
    pub items: &'static [&'static str],
}

pub const MARKDOWN_CHEATSHEET_SECTIONS: &[MarkdownCheatsheetSection] = &[
    MarkdownCheatsheetSection {
        title: "Headings",
        items: &[
            "# Heading 1",
            "## Heading 2",
            "### Heading 3",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Emphasis",
        items: &["**bold**", "*italic*", "~~strikethrough~~"],
    },
    MarkdownCheatsheetSection {
        title: "Lists",
        items: &["- Unordered item", "1. Ordered item", "- [ ] Task"],
    },
    MarkdownCheatsheetSection {
        title: "Links & code",
        items: &[
            "[Link text](https://example.com)",
            "`inline code`",
            "```\ncode block\n```",
        ],
    },
];

pub fn markdown_cheatsheet_text() -> String {
    MARKDOWN_CHEATSHEET_SECTIONS
        .iter()
        .map(|section| {
            format!(
                "{}\n{}",
                section.title,
                section
                    .items
                    .iter()
                    .map(|item| format!("  {item}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn apply_markdown_command(
    content: &str,
    selection: ByteSelection,
    command: MarkdownCommand,
) -> FormattingResult {
    let (prefix, suffix) = command.affixes();
    apply_markdown_format(content, selection, prefix, suffix)
}

#[cfg(test)]
mod cheatsheet_tests {
    use super::*;

    #[test]
    fn markdown_cheatsheet_includes_shared_syntax_sections() {
        let text = markdown_cheatsheet_text();
        assert!(text.contains("Emphasis"));
        assert!(text.contains("**bold**"));
        assert!(text.contains("- [ ] Task"));
    }
}

pub fn apply_markdown_format(
    content: &str,
    selection: ByteSelection,
    prefix: &str,
    suffix: &str,
) -> FormattingResult {
    let (start, end) = selection.ordered();
    let formatted = format_text(content, start, end, prefix, suffix);
    FormattingResult {
        content: formatted,
        caret_byte: end + prefix.len() + suffix.len(),
    }
}

pub fn format_text(content: &str, start: usize, end: usize, prefix: &str, suffix: &str) -> String {
    let start = clamp_to_char_boundary(content, start);
    let end = clamp_to_char_boundary(content, end);
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    let mut result = String::with_capacity(content.len() + prefix.len() + suffix.len());
    result.push_str(&content[..start]);
    result.push_str(prefix);
    result.push_str(&content[start..end]);
    result.push_str(suffix);
    result.push_str(&content[end..]);
    result
}

fn clamp_to_char_boundary(content: &str, index: usize) -> usize {
    let mut clamped = index.min(content.len());
    while clamped > 0 && !content.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_requires_a_valid_utf8_byte_range() {
        assert!(ByteSelection::new("A😀B", 1, 5).is_some());
        assert!(ByteSelection::new("A😀B", 2, 5).is_none());
    }

    #[test]
    fn formatting_unicode_returns_a_byte_caret() {
        let selection = ByteSelection::new("A😀B", 1, 5).unwrap();
        let result = apply_markdown_command("A😀B", selection, MarkdownCommand::Bold);

        assert_eq!(result.content, "A**😀**B");
        assert_eq!(result.caret_byte, 9);
    }
}
