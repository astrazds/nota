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
            "#### Heading 4",
            "##### Heading 5",
            "###### Heading 6",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Emphasis",
        items: &[
            "**bold** or __bold__",
            "*italic* or _italic_",
            "***bold italic***",
            "~~strikethrough~~",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Lists",
        items: &[
            "- Unordered item",
            "* Also unordered",
            "1. Ordered item",
            "   - Nested item",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Task Lists",
        items: &["- [ ] To do", "- [x] Done"],
    },
    MarkdownCheatsheetSection {
        title: "Links & Images",
        items: &[
            "[Link text](https://example.com)",
            "<https://example.com>",
            "![Alt text](https://example.com/image.png)",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Code",
        items: &[
            "`inline code`",
            "```rust",
            "fn main() { println!(\"hi\"); }",
            "```",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Quotes & Rules",
        items: &["> Blockquote", "> Nested quote", "--- (horizontal rule)"],
    },
    MarkdownCheatsheetSection {
        title: "Tables",
        items: &["| Name | Value |", "| --- | --- |", "| Foo | Bar |"],
    },
    MarkdownCheatsheetSection {
        title: "Footnotes",
        items: &["Reference[^1]", "[^1]: Footnote text"],
    },
    MarkdownCheatsheetSection {
        title: "Line Breaks",
        items: &[
            "End line with two spaces  ",
            "or use a blank line between paragraphs",
        ],
    },
    MarkdownCheatsheetSection {
        title: "Escaping",
        items: &["\\*literal asterisks\\*", "\\# literal heading marker"],
    },
    MarkdownCheatsheetSection {
        title: "Safety",
        items: &["Raw HTML is displayed as text for safety."],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserSelection {
    pub start_utf16: usize,
    pub end_utf16: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattingResult {
    pub content: String,
    pub caret_utf16: usize,
}

pub fn apply_markdown_command(
    content: &str,
    selection: BrowserSelection,
    command: MarkdownCommand,
) -> FormattingResult {
    let (prefix, suffix) = command.affixes();
    apply_markdown_format(content, selection, prefix, suffix)
}

pub fn apply_markdown_format(
    content: &str,
    selection: BrowserSelection,
    prefix: &str,
    suffix: &str,
) -> FormattingResult {
    let (selection_start_utf16, selection_end_utf16) =
        if selection.start_utf16 <= selection.end_utf16 {
            (selection.start_utf16, selection.end_utf16)
        } else {
            (selection.end_utf16, selection.start_utf16)
        };
    let (start, end) =
        utf16_range_to_byte_range(content, selection_start_utf16, selection_end_utf16);
    let formatted = format_text(content, start, end, prefix, suffix);
    let caret_utf16 = selection_start_utf16
        + prefix.encode_utf16().count()
        + (selection_end_utf16 - selection_start_utf16)
        + suffix.encode_utf16().count();

    FormattingResult {
        content: formatted,
        caret_utf16,
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

    let selected_text = &content[start..end];
    let mut result = String::with_capacity(content.len() + prefix.len() + suffix.len());
    result.push_str(&content[..start]);
    result.push_str(prefix);
    result.push_str(selected_text);
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

pub fn utf16_index_to_byte_index(content: &str, utf16_index: usize) -> usize {
    if utf16_index == 0 {
        return 0;
    }

    let mut utf16_count = 0;
    for (byte_index, ch) in content.char_indices() {
        if utf16_count >= utf16_index {
            return byte_index;
        }

        let next_utf16_count = utf16_count + ch.len_utf16();
        if next_utf16_count > utf16_index {
            return byte_index;
        }

        utf16_count = next_utf16_count;
    }

    content.len()
}

pub fn utf16_range_to_byte_range(
    content: &str,
    start_utf16: usize,
    end_utf16: usize,
) -> (usize, usize) {
    let start_byte = utf16_index_to_byte_index(content, start_utf16);
    let end_byte = utf16_index_to_byte_index(content, end_utf16);
    if start_byte <= end_byte {
        (start_byte, end_byte)
    } else {
        (end_byte, start_byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_named_markdown_commands() {
        let bold = apply_markdown_command(
            "Hello world",
            BrowserSelection {
                start_utf16: 6,
                end_utf16: 11,
            },
            MarkdownCommand::Bold,
        );
        assert_eq!(bold.content, "Hello **world**");

        let table = apply_markdown_command(
            "Hello",
            BrowserSelection {
                start_utf16: 5,
                end_utf16: 5,
            },
            MarkdownCommand::Table,
        );
        assert!(table.content.contains("| Column 1 | Column 2 |"));
    }

    #[test]
    fn formats_browser_selection_without_corrupting_unicode_and_returns_caret() {
        let result = apply_markdown_format(
            "A😀B",
            BrowserSelection {
                start_utf16: 1,
                end_utf16: 3,
            },
            "**",
            "**",
        );

        assert_eq!(result.content, "A**😀**B");
        assert_eq!(result.caret_utf16, 7);
    }

    #[test]
    fn inserts_formatting_for_empty_selection() {
        let result = apply_markdown_format(
            "Hello world",
            BrowserSelection {
                start_utf16: 5,
                end_utf16: 5,
            },
            "**",
            "**",
        );

        assert_eq!(result.content, "Hello**** world");
        assert_eq!(result.caret_utf16, 9);
    }

    #[test]
    fn clamps_utf16_index_inside_surrogate_pair() {
        let content = "😀a";
        let byte_index = utf16_index_to_byte_index(content, 1);
        assert_eq!(byte_index, 0);
    }
}
