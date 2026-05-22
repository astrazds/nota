use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    #[serde(default = "default_created")]
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_pinned: bool,
}

fn default_created() -> DateTime<Utc> {
    Utc::now()
}

impl Note {
    pub fn new(title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            created: now,
            last_modified: now,
            tags: Vec::new(),
            is_pinned: false,
        }
    }

    pub fn display_title(&self) -> &str {
        let trimmed = self.title.trim();
        if trimmed.is_empty() {
            "New Note"
        } else {
            trimmed
        }
    }

    pub fn display_date(&self) -> String {
        self.last_modified.format("%d/%m/%Y").to_string()
    }

    pub fn preview(&self) -> String {
        let preview = self.content.lines().next().unwrap_or("No additional text");
        if preview.chars().count() > 50 {
            let truncated: String = preview.chars().take(47).collect();
            format!("{truncated}...")
        } else {
            preview.to_string()
        }
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn character_count(&self) -> usize {
        self.content.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_editing::{
        format_text, utf16_index_to_byte_index, utf16_range_to_byte_range,
    };
    use crate::note_discovery::project_note_list;
    use crate::tag_rules::parse_tags_input;

    fn projected_ids(notes: &[Note], query: &str, active_tag: Option<&str>) -> Vec<Uuid> {
        project_note_list(notes, None, query, active_tag)
            .rows
            .into_iter()
            .map(|row| row.id)
            .collect()
    }

    #[test]
    fn should_project_and_sort_notes_correctly() {
        let mut n1 = Note::new("Apple".to_string(), "Fruit".to_string());
        let mut n2 = Note::new("Banana".to_string(), "Yellow".to_string());
        let mut n3 = Note::new("Cherry".to_string(), "Red".to_string());

        // Set specific modification times for sorting
        n1.last_modified = Utc::now();
        n2.last_modified = Utc::now() + chrono::Duration::seconds(10);
        n3.last_modified = Utc::now() + chrono::Duration::seconds(20);

        let notes = vec![n1.clone(), n2.clone(), n3.clone()];

        // Default sort (newest first)
        let result = projected_ids(&notes, "", None);
        assert_eq!(result[0], n3.id);
        assert_eq!(result[1], n2.id);
        assert_eq!(result[2], n1.id);

        // Search filter
        let result = projected_ids(&notes, "ba", None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], n2.id);

        // Search content
        let result = projected_ids(&notes, "red", None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], n3.id);

        // Pinning (pinned first)
        let mut pinned_n1 = n1.clone();
        pinned_n1.is_pinned = true;
        let notes_with_pin = vec![pinned_n1.clone(), n2.clone(), n3.clone()];
        let result = projected_ids(&notes_with_pin, "", None);
        assert_eq!(result[0], pinned_n1.id); // Pinned is first even if older
        assert_eq!(result[1], n3.id);
    }

    #[test]
    fn should_calculate_counts_correctly() {
        let note = Note::new("".to_string(), "Hello world, this is a test.".to_string());
        assert_eq!(note.word_count(), 6);
        assert_eq!(note.character_count(), 28);

        let empty_note = Note::new("".to_string(), "".to_string());
        assert_eq!(empty_note.word_count(), 0);
        assert_eq!(empty_note.character_count(), 0);

        // Markdown complexity
        let md_note = Note::new(
            "".to_string(),
            "# Title\n\n- List item\n- List item 2".to_string(),
        );
        assert_eq!(md_note.word_count(), 9);

        // Unicode characters should be counted correctly (not bytes)
        let unicode_note = Note::new("".to_string(), "日本語".to_string());
        assert_eq!(unicode_note.character_count(), 3); // 3 chars, 9 bytes in UTF-8
        assert_eq!(unicode_note.word_count(), 1);
    }

    #[test]
    fn should_format_text_correctly() {
        let content = "Hello world";

        // Bold "world"
        let result = format_text(content, 6, 11, "**", "**");
        assert_eq!(result, "Hello **world**");

        // Bold "Hello"
        let result = format_text(content, 0, 5, "**", "**");
        assert_eq!(result, "**Hello** world");

        // Link "Hello"
        let result = format_text(content, 0, 5, "[", "](https://google.com)");
        assert_eq!(result, "[Hello](https://google.com) world");

        // Empty selection (just insert)
        let result = format_text(content, 5, 5, "**", "**");
        assert_eq!(result, "Hello**** world");

        // At the very end
        let result = format_text(content, 11, 11, "!", "!");
        assert_eq!(result, "Hello world!!");
    }

    #[test]
    fn should_format_text_with_invalid_indices_safely() {
        let content = "A😀B";

        let result = format_text(content, 2, 5, "**", "**");
        assert_eq!(result, "A**😀**B");

        let result = format_text(content, 999, 1000, "[", "]");
        assert_eq!(result, "A😀B[]");

        let result = format_text(content, 5, 1, "(", ")");
        assert_eq!(result, "A(😀)B");
    }

    #[test]
    fn should_create_note_with_correct_fields() {
        let title = "Test Title".to_string();
        let content = "Test Content".to_string();
        let note = Note::new(title.clone(), content.clone());

        assert_eq!(note.title, title);
        assert_eq!(note.content, content);
        assert!(!note.id.is_nil());
        assert!(note.tags.is_empty());
        assert!(!note.is_pinned);
        assert!(note.created <= Utc::now());
        assert_eq!(note.created, note.last_modified);
    }

    #[test]
    fn should_deserialise_existing_notes_without_created_field() {
        #[derive(Serialize, Deserialize)]
        struct OldNoteFormat {
            pub id: Uuid,
            pub title: String,
            pub content: String,
            pub last_modified: DateTime<Utc>,
            pub is_pinned: bool,
        }

        let old_note = OldNoteFormat {
            id: Uuid::new_v4(),
            title: "Test".to_string(),
            content: "Content".to_string(),
            last_modified: Utc::now(),
            is_pinned: false,
        };

        let json = serde_json::to_string(&old_note).unwrap();
        let note: Note = serde_json::from_str(&json).unwrap();

        assert_eq!(note.title, "Test");
        assert_eq!(note.content, "Content");
        assert!(note.tags.is_empty());
        assert!(note.created <= Utc::now());
    }

    #[test]
    fn should_return_correct_display_title() {
        let mut note = Note::new("".to_string(), "".to_string());
        assert_eq!(note.display_title(), "New Note");

        note.title = "   ".to_string();
        assert_eq!(note.display_title(), "New Note");

        note.title = "Real Title".to_string();
        assert_eq!(note.display_title(), "Real Title");

        note.title = "A".to_string();
        assert_eq!(note.display_title(), "A");
    }

    #[test]
    fn should_return_correct_preview() {
        let mut note = Note::new("".to_string(), "Line 1\nLine 2".to_string());
        assert_eq!(note.preview(), "Line 1");

        note.content = "".to_string();
        assert_eq!(note.preview(), "No additional text");

        note.content =
            "A very long line that exceeds the fifty character limit of the preview function"
                .to_string();
        assert_eq!(
            note.preview(),
            "A very long line that exceeds the fifty charact..."
        );

        note.content = "Short line".to_string();
        assert_eq!(note.preview(), "Short line");

        note.content = "Line with exactly 50 characters to test boundary.".to_string();
        assert_eq!(
            note.preview(),
            "Line with exactly 50 characters to test boundary."
        );

        note.content = "Line with exactly 51 characters to test boundary...".to_string();
        assert_eq!(
            note.preview(),
            "Line with exactly 51 characters to test boundar..."
        );

        note.content = "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀".to_string();
        assert_eq!(
            note.preview(),
            "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"
        );

        note.content = "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀".to_string();
        assert_eq!(
            note.preview(),
            "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"
        );

        note.content = "あ".repeat(51);
        assert!(note.preview().ends_with("..."));
        assert_eq!(note.preview().chars().count(), 50);
    }

    #[test]
    fn should_format_date_correctly() {
        let note = Note::new("".to_string(), "".to_string());
        let date_str = note.display_date();
        // Check format DD/MM/YYYY
        assert!(date_str.chars().nth(2) == Some('/'));
        assert!(date_str.chars().nth(5) == Some('/'));
        assert_eq!(date_str.len(), 10);
    }

    #[test]
    fn should_filter_unicode_notes() {
        let notes = vec![
            Note::new("日本語".to_string(), "内容".to_string()),
            Note::new("Hello".to_string(), "World".to_string()),
        ];
        let result = projected_ids(&notes, "日本", None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], notes[0].id);
    }

    #[test]
    fn should_filter_empty_notes() {
        let notes = vec![Note::new("".to_string(), "".to_string())];
        let result = projected_ids(&notes, "", None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn should_handle_special_characters_in_search() {
        let notes = vec![Note::new("Test @#$%".to_string(), "Content".to_string())];
        let result = projected_ids(&notes, "@#$", None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn should_handle_empty_notes_vector() {
        let notes: Vec<Note> = vec![];
        let result = projected_ids(&notes, "anything", None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_search_case_insensitive() {
        let notes = vec![Note::new("HELLO world".to_string(), "Content".to_string())];
        let result = projected_ids(&notes, "hello", None);
        assert_eq!(result.len(), 1);
        let result = projected_ids(&notes, "HELLO", None);
        assert_eq!(result.len(), 1);
        let result = projected_ids(&notes, "HeLLo", None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn should_parse_tags_input() {
        let tags = parse_tags_input(" work, rust,work, , Product ");
        assert_eq!(
            tags,
            vec![
                "work".to_string(),
                "rust".to_string(),
                "Product".to_string()
            ]
        );
    }

    #[test]
    fn should_filter_notes_by_active_tag() {
        let mut n1 = Note::new("One".to_string(), "First".to_string());
        n1.tags = vec!["Work".to_string(), "Rust".to_string()];

        let mut n2 = Note::new("Two".to_string(), "Second".to_string());
        n2.tags = vec!["Personal".to_string()];

        let notes = vec![n1.clone(), n2.clone()];
        let result = projected_ids(&notes, "", Some("work"));
        assert_eq!(result, vec![n1.id]);

        let result = projected_ids(&notes, "", Some("PERSONAL"));
        assert_eq!(result, vec![n2.id]);
    }

    #[test]
    fn should_search_tags_with_query() {
        let mut n1 = Note::new("One".to_string(), "First".to_string());
        n1.tags = vec!["Project".to_string()];

        let n2 = Note::new("Two".to_string(), "Second".to_string());

        let notes = vec![n1.clone(), n2];
        let result = projected_ids(&notes, "pro", None);
        assert_eq!(result, vec![n1.id]);
    }

    #[test]
    fn should_convert_utf16_ranges_to_safe_utf8_boundaries() {
        let content = "A😀B";
        // UTF-16 layout: A(1), 😀(2), B(1)
        let (start, end) = utf16_range_to_byte_range(content, 1, 3);
        assert_eq!(&content[start..end], "😀");

        let (start, end) = utf16_range_to_byte_range(content, 3, 4);
        assert_eq!(&content[start..end], "B");
    }

    #[test]
    fn should_clamp_utf16_index_inside_surrogate_pair() {
        let content = "😀a";
        let byte_index = utf16_index_to_byte_index(content, 1);
        assert_eq!(byte_index, 0);
    }
}
