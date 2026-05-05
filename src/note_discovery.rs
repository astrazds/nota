use crate::model::Note;
use crate::tag_rules::{fold_case, note_has_active_tag};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HighlightSegment {
    pub text: String,
    pub is_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteListRenderKey {
    pub id: Uuid,
    pub display_title: String,
    pub display_date: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub is_selected: bool,
    pub title_highlights: Vec<HighlightSegment>,
    pub preview_highlights: Vec<HighlightSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteListItem {
    pub id: Uuid,
    pub display_title: String,
    pub display_date: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub is_selected: bool,
    pub title_highlights: Vec<HighlightSegment>,
    pub preview_highlights: Vec<HighlightSegment>,
}

impl NoteListItem {
    pub fn render_key(&self) -> NoteListRenderKey {
        NoteListRenderKey {
            id: self.id,
            display_title: self.display_title.clone(),
            display_date: self.display_date.clone(),
            preview: self.preview.clone(),
            tags: self.tags.clone(),
            is_pinned: self.is_pinned,
            is_selected: self.is_selected,
            title_highlights: self.title_highlights.clone(),
            preview_highlights: self.preview_highlights.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteListProjection {
    pub rows: Vec<NoteListItem>,
    pub has_active_filter: bool,
}

pub fn project_note_list(
    notes: &[Note],
    selected_id: Option<Uuid>,
    query: &str,
    active_tag: Option<&str>,
) -> NoteListProjection {
    let rows = filter_and_sort_notes(notes, query, active_tag)
        .into_iter()
        .filter_map(|id| notes.iter().find(|note| note.id == id))
        .map(|note| {
            let display_title = note.display_title().to_string();
            let preview = note.preview();
            NoteListItem {
                id: note.id,
                display_date: note.display_date(),
                tags: note.tags.clone(),
                is_pinned: note.is_pinned,
                is_selected: selected_id == Some(note.id),
                title_highlights: highlight_segments(&display_title, query),
                preview_highlights: highlight_segments(&preview, query),
                display_title,
                preview,
            }
        })
        .collect();

    NoteListProjection {
        rows,
        has_active_filter: !query.is_empty() || active_tag.is_some(),
    }
}

pub fn filter_and_sort_notes(notes: &[Note], query: &str, active_tag: Option<&str>) -> Vec<Uuid> {
    let folded_query = build_folded_query(query);
    let active_tag = active_tag.map(str::trim).filter(|tag| !tag.is_empty());
    let mut filtered: Vec<&Note> = notes
        .iter()
        .filter(|note| {
            let matches_query = contains_folded_query(&note.title, &folded_query)
                || contains_folded_query(&note.content, &folded_query)
                || note
                    .tags
                    .iter()
                    .any(|tag| contains_folded_query(tag, &folded_query));

            let matches_active_tag =
                active_tag.is_none_or(|active| note_has_active_tag(note, active));

            matches_query && matches_active_tag
        })
        .collect();

    filtered.sort_by(|a, b| {
        b.is_pinned
            .cmp(&a.is_pinned)
            .then_with(|| b.last_modified.cmp(&a.last_modified))
    });

    filtered.into_iter().map(|note| note.id).collect()
}

pub fn highlight_segments(text: &str, query: &str) -> Vec<HighlightSegment> {
    let match_ranges = find_case_insensitive_match_ranges(text, query);
    if match_ranges.is_empty() {
        return vec![HighlightSegment {
            text: text.to_string(),
            is_match: false,
        }];
    }

    let mut segments = Vec::with_capacity(match_ranges.len() * 2 + 1);
    let mut last_end = 0;

    for (start, end) in match_ranges {
        if start > last_end {
            segments.push(HighlightSegment {
                text: text[last_end..start].to_string(),
                is_match: false,
            });
        }
        segments.push(HighlightSegment {
            text: text[start..end].to_string(),
            is_match: true,
        });
        last_end = end;
    }

    if last_end < text.len() {
        segments.push(HighlightSegment {
            text: text[last_end..].to_string(),
            is_match: false,
        });
    }

    segments
}

enum FoldedQuery {
    Empty,
    Ascii(String),
    Unicode(String),
}

fn build_folded_query(query: &str) -> FoldedQuery {
    if query.is_empty() {
        FoldedQuery::Empty
    } else if query.is_ascii() {
        FoldedQuery::Ascii(query.to_ascii_lowercase())
    } else {
        FoldedQuery::Unicode(query.to_lowercase())
    }
}

fn contains_folded_query(text: &str, query: &FoldedQuery) -> bool {
    match query {
        FoldedQuery::Empty => true,
        FoldedQuery::Ascii(q) => {
            if text.is_ascii() {
                text.to_ascii_lowercase().contains(q)
            } else {
                text.to_lowercase().contains(q)
            }
        }
        FoldedQuery::Unicode(q) => text.to_lowercase().contains(q),
    }
}

fn find_case_insensitive_match_ranges(text: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }

    if text.is_ascii() && query.is_ascii() {
        let text_lower = text.to_ascii_lowercase();
        let query_lower = query.to_ascii_lowercase();
        let mut ranges = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = text_lower[search_start..].find(&query_lower) {
            let start = search_start + pos;
            let end = start + query_lower.len();
            ranges.push((start, end));
            search_start = end;

            if search_start >= text_lower.len() {
                break;
            }
        }

        return ranges;
    }

    find_case_insensitive_match_ranges_unicode(text, query)
}

fn find_case_insensitive_match_ranges_unicode(text: &str, query: &str) -> Vec<(usize, usize)> {
    let query_lower = query.to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }

    let mut boundaries: Vec<usize> = text.char_indices().map(|(idx, _)| idx).collect();
    boundaries.push(text.len());

    let query_lower_len = query_lower.len();
    let mut ranges = Vec::new();
    let mut i = 0;

    while i + 1 < boundaries.len() {
        let start = boundaries[i];
        let mut matched_end = None;

        for &end in &boundaries[i + 1..] {
            let candidate_lower = fold_case(&text[start..end]);
            if candidate_lower.starts_with(&query_lower) {
                matched_end = Some(end);
                break;
            }

            if candidate_lower.len() >= query_lower_len {
                break;
            }
        }

        if let Some(end) = matched_end {
            ranges.push((start, end));
            if let Ok(next_index) = boundaries.binary_search(&end) {
                i = next_index;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn discovers_notes_with_filter_order_and_highlights_using_one_query() {
        let mut older_pinned = Note::new("Rust Work".to_string(), "Pinned note".to_string());
        older_pinned.tags = vec!["Work".to_string()];
        older_pinned.is_pinned = true;
        older_pinned.last_modified = Utc::now();

        let mut newer_unpinned = Note::new("Other".to_string(), "rust content".to_string());
        newer_unpinned.tags = vec!["Work".to_string()];
        newer_unpinned.last_modified = older_pinned.last_modified + chrono::Duration::seconds(10);

        let notes = vec![newer_unpinned.clone(), older_pinned.clone()];

        let ids = filter_and_sort_notes(&notes, "rust", Some("work"));
        assert_eq!(ids, vec![older_pinned.id, newer_unpinned.id]);

        let highlighted = highlight_segments("Rust Work", "rust");
        assert_eq!(
            highlighted,
            vec![
                HighlightSegment {
                    text: "Rust".to_string(),
                    is_match: true,
                },
                HighlightSegment {
                    text: " Work".to_string(),
                    is_match: false,
                },
            ]
        );
    }

    #[test]
    fn projects_render_ready_note_rows() {
        let mut older_pinned = Note::new("Rust Work".to_string(), "Pinned note".to_string());
        older_pinned.tags = vec!["Work".to_string()];
        older_pinned.is_pinned = true;
        older_pinned.last_modified = Utc::now();

        let mut newer_unpinned = Note::new("Other".to_string(), "rust content".to_string());
        newer_unpinned.tags = vec!["Work".to_string()];
        newer_unpinned.last_modified = older_pinned.last_modified + chrono::Duration::seconds(10);

        let projection = project_note_list(
            &[newer_unpinned.clone(), older_pinned.clone()],
            Some(older_pinned.id),
            "rust",
            Some("work"),
        );

        assert!(projection.has_active_filter);
        assert_eq!(projection.rows.len(), 2);
        assert_eq!(projection.rows[0].id, older_pinned.id);
        assert!(projection.rows[0].is_selected);
        assert!(projection.rows[0].is_pinned);
        assert_eq!(projection.rows[0].display_title, "Rust Work");
        assert_eq!(projection.rows[0].preview, "Pinned note");
        assert!(
            projection.rows[0]
                .title_highlights
                .iter()
                .any(|segment| segment.is_match)
        );
        assert_eq!(projection.rows[1].id, newer_unpinned.id);
    }

    #[test]
    fn row_render_key_changes_when_displayed_note_fields_change() {
        let mut note = Note::new("Draft".to_string(), "Preview".to_string());
        note.tags = vec!["work".to_string()];

        let before = project_note_list(&[note.clone()], Some(note.id), "", None)
            .rows
            .remove(0)
            .render_key();

        note.title = "Published".to_string();
        note.content = "Updated preview".to_string();
        note.tags = vec!["personal".to_string()];

        let after = project_note_list(&[note], None, "", None)
            .rows
            .remove(0)
            .render_key();

        assert_ne!(before, after);
    }

    #[test]
    fn highlights_unicode_case_folded_matches() {
        let highlighted = highlight_segments("İstanbul", "i");
        assert_eq!(
            highlighted,
            vec![
                HighlightSegment {
                    text: "İ".to_string(),
                    is_match: true,
                },
                HighlightSegment {
                    text: "stanbul".to_string(),
                    is_match: false,
                },
            ]
        );
    }
}
