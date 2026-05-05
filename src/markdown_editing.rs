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
