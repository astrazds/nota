use crate::model::Note;
#[cfg(debug_assertions)]
use chrono::{DateTime, Utc};
#[cfg(debug_assertions)]
use uuid::Uuid;

pub fn debug_starter_notes() -> Vec<Note> {
    debug_starter_notes_for_build()
}

#[cfg(debug_assertions)]
fn debug_starter_notes_for_build() -> Vec<Note> {
    vec![
        Note {
            id: note_id("11111111-1111-4111-8111-111111111111"),
            title: "Pinned launch checklist".to_string(),
            content: r#"# Pinned launch checklist

Keep this note pinned while testing note ordering, note actions, and task-list rendering.

- [x] Verify **bold** toolbar output
- [x] Verify _italic_ toolbar output
- [ ] Try ~~strikethrough~~ on completed details
- [ ] Toggle the sidebar and confirm this title stays selected

Related link: [Nota repository](https://repos.astrazds.net/astrazds/nota)
"#
            .trim()
            .to_string(),
            created: timestamp("2026-05-01T08:00:00Z"),
            last_modified: timestamp("2026-05-03T10:15:00Z"),
            tags: vec!["Work".to_string(), "UX".to_string()],
            is_pinned: true,
        },
        Note {
            id: note_id("22222222-2222-4222-8222-222222222222"),
            title: "Markdown preview tour".to_string(),
            content: r#"# Markdown preview tour

Use this note to compare Write, Preview, and Split modes with richer Markdown.

| Feature | Expected result |
| --- | --- |
| Table | Aligned rows in preview |
| `inline code` | Monospace text |
| Footnote[^1] | Linked footnote section |

```rust
fn preview_mode() -> &'static str {
    "consistent padding"
}
```

[^1]: Footnotes should render at the bottom without unsafe HTML.
"#
            .trim()
            .to_string(),
            created: timestamp("2026-05-01T09:00:00Z"),
            last_modified: timestamp("2026-05-02T14:30:00Z"),
            tags: vec!["Markdown".to_string(), "Reference".to_string()],
            is_pinned: false,
        },
        Note {
            id: note_id("33333333-3333-4333-8333-333333333333"),
            title: "Responsive editing regression note with a deliberately long title".to_string(),
            content: r#"This first line is intentionally long so the note list preview truncates cleanly while search, tags, and responsive layouts stay readable.

## Mobile checks

1. Open the note list.
2. Filter by the `Mobile` tag.
3. Edit the title, body, and tags without losing focus.

HTML should stay escaped: <script>alert("noter")</script>
"#
            .trim()
            .to_string(),
            created: timestamp("2026-05-01T10:00:00Z"),
            last_modified: timestamp("2026-05-01T16:45:00Z"),
            tags: vec!["Personal".to_string(), "Mobile".to_string()],
            is_pinned: false,
        },
    ]
}

#[cfg(not(debug_assertions))]
fn debug_starter_notes_for_build() -> Vec<Note> {
    Vec::new()
}

#[cfg(debug_assertions)]
fn note_id(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("starter note IDs must be valid UUIDs")
}

#[cfg(debug_assertions)]
fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("starter note timestamps must be valid RFC 3339 values")
        .with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_starter_notes_cover_core_ui_behaviours() {
        let notes = debug_starter_notes();

        assert_eq!(notes.len(), 3);
        assert!(notes.iter().any(|note| note.is_pinned));
        assert!(
            notes
                .iter()
                .flat_map(|note| note.tags.iter())
                .any(|tag| tag == "Mobile")
        );

        let combined_content = notes
            .iter()
            .map(|note| note.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        for expected in [
            "**bold**",
            "_italic_",
            "~~strikethrough~~",
            "- [ ]",
            "| Feature | Expected result |",
            "Footnote[^1]",
            "[Nota repository]",
            "<script>",
        ] {
            assert!(
                combined_content.contains(expected),
                "starter notes should include {expected}"
            );
        }
    }
}
