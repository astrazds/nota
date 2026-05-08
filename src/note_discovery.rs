use crate::model::Note;
use crate::search_query::SearchQuery;
use crate::tag_rules::{fold_case, note_has_active_tag};
use uuid::Uuid;

const MATCH_SNIPPET_CONTEXT_CHARS: usize = 24;

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
    pub tag_highlights: Vec<Vec<HighlightSegment>>,
    pub uses_match_snippet: bool,
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
    pub tag_highlights: Vec<Vec<HighlightSegment>>,
    pub uses_match_snippet: bool,
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
            tag_highlights: self.tag_highlights.clone(),
            uses_match_snippet: self.uses_match_snippet,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteListProjection {
    pub rows: Vec<NoteListItem>,
    pub has_active_filter: bool,
    pub selected_note_visibility: SelectedNoteVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedNoteVisibility {
    NoSelection,
    Visible,
    HiddenByFilter,
}

pub fn project_note_list(
    notes: &[Note],
    selected_id: Option<Uuid>,
    query: &str,
    active_tag: Option<&str>,
) -> NoteListProjection {
    let search_query = SearchQuery::parse(query);
    let title_highlight_terms = search_query.title_highlight_terms();
    let preview_highlight_terms = search_query.preview_highlight_terms();
    let tag_highlight_terms = search_query.tag_highlight_terms();
    let has_active_filter =
        !search_query.is_empty() || active_tag.map(str::trim).is_some_and(|tag| !tag.is_empty());
    let rows: Vec<NoteListItem> =
        filter_and_sort_notes_with_query(notes, &search_query, active_tag)
            .into_iter()
            .filter_map(|id| notes.iter().find(|note| note.id == id))
            .map(|note| {
                let display_title = note.display_title().to_string();
                let title_highlights =
                    highlight_segments_for_terms(&display_title, &title_highlight_terms);
                let tag_highlights: Vec<Vec<HighlightSegment>> = note
                    .tags
                    .iter()
                    .map(|tag| highlight_segments_for_terms(tag, &tag_highlight_terms))
                    .collect();
                let title_explains_match = title_highlights.iter().any(|segment| segment.is_match);
                let tags_explain_match = tag_highlights
                    .iter()
                    .flatten()
                    .any(|segment| segment.is_match);
                let (preview, preview_highlights, uses_match_snippet) =
                    if title_explains_match || tags_explain_match {
                        let preview = note.preview();
                        let preview_highlights =
                            highlight_segments_for_terms(&preview, &preview_highlight_terms);
                        (preview, preview_highlights, false)
                    } else if let Some(snippet) =
                        match_snippet_for_terms(&note.content, &preview_highlight_terms)
                    {
                        snippet
                    } else {
                        let preview = note.preview();
                        let preview_highlights =
                            highlight_segments_for_terms(&preview, &preview_highlight_terms);
                        (preview, preview_highlights, false)
                    };
                NoteListItem {
                    id: note.id,
                    display_date: note.display_date(),
                    tags: note.tags.clone(),
                    is_pinned: note.is_pinned,
                    is_selected: selected_id == Some(note.id),
                    title_highlights,
                    preview_highlights,
                    tag_highlights,
                    uses_match_snippet,
                    display_title,
                    preview,
                }
            })
            .collect();
    let selected_note_visibility =
        selected_note_visibility(notes, selected_id, &rows, has_active_filter);

    NoteListProjection {
        rows,
        has_active_filter,
        selected_note_visibility,
    }
}

fn selected_note_visibility(
    notes: &[Note],
    selected_id: Option<Uuid>,
    rows: &[NoteListItem],
    has_active_filter: bool,
) -> SelectedNoteVisibility {
    let Some(selected_id) = selected_id else {
        return SelectedNoteVisibility::NoSelection;
    };

    if rows.iter().any(|row| row.id == selected_id) {
        return SelectedNoteVisibility::Visible;
    }

    if has_active_filter && notes.iter().any(|note| note.id == selected_id) {
        SelectedNoteVisibility::HiddenByFilter
    } else {
        SelectedNoteVisibility::NoSelection
    }
}

fn filter_and_sort_notes_with_query(
    notes: &[Note],
    query: &SearchQuery,
    active_tag: Option<&str>,
) -> Vec<Uuid> {
    let active_tag = active_tag.map(str::trim).filter(|tag| !tag.is_empty());
    let mut filtered: Vec<&Note> = notes
        .iter()
        .filter(|note| {
            let matches_query = query.matches(note);

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

fn highlight_segments_for_terms(text: &str, terms: &[&str]) -> Vec<HighlightSegment> {
    let match_ranges = find_case_insensitive_match_ranges_for_terms(text, terms);
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

fn find_case_insensitive_match_ranges_for_terms(text: &str, terms: &[&str]) -> Vec<(usize, usize)> {
    let mut ranges: Vec<(usize, usize)> = terms
        .iter()
        .flat_map(|term| find_case_insensitive_match_ranges(text, term))
        .collect();
    ranges.sort_by(|(left_start, left_end), (right_start, right_end)| {
        left_start
            .cmp(right_start)
            .then_with(|| right_end.cmp(left_end))
    });

    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some((_, last_end)) if start <= *last_end => {
                *last_end = (*last_end).max(end);
            }
            _ => merged.push((start, end)),
        }
    }

    merged
}

fn match_snippet_for_terms(
    text: &str,
    terms: &[&str],
) -> Option<(String, Vec<HighlightSegment>, bool)> {
    let source = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if source.is_empty() || terms.is_empty() {
        return None;
    }

    let ranges = find_case_insensitive_match_ranges_for_terms(&source, terms);
    let (match_start, match_end) = best_match_window_anchor(&source, terms, &ranges)?;

    let boundaries = char_boundaries(&source);
    let start_char =
        byte_to_char_index(&boundaries, match_start).saturating_sub(MATCH_SNIPPET_CONTEXT_CHARS);
    let end_char = (byte_to_char_index(&boundaries, match_end) + MATCH_SNIPPET_CONTEXT_CHARS)
        .min(boundaries.len().saturating_sub(1));
    let start = boundaries[start_char];
    let end = boundaries[end_char];
    let clipped_start = start > 0;
    let clipped_end = end < source.len();
    let mut snippet = String::new();
    if clipped_start {
        snippet.push_str("...");
    }
    snippet.push_str(source[start..end].trim());
    if clipped_end {
        snippet.push_str("...");
    }
    let highlights = highlight_segments_for_terms(&snippet, terms);

    Some((snippet, highlights, true))
}

fn best_match_window_anchor(
    source: &str,
    terms: &[&str],
    ranges: &[(usize, usize)],
) -> Option<(usize, usize)> {
    ranges.iter().copied().max_by_key(|(start, end)| {
        let boundaries = char_boundaries(source);
        let start_char =
            byte_to_char_index(&boundaries, *start).saturating_sub(MATCH_SNIPPET_CONTEXT_CHARS);
        let end_char = (byte_to_char_index(&boundaries, *end) + MATCH_SNIPPET_CONTEXT_CHARS)
            .min(boundaries.len().saturating_sub(1));
        let window = &source[boundaries[start_char]..boundaries[end_char]];
        terms
            .iter()
            .filter(|term| !find_case_insensitive_match_ranges(window, term).is_empty())
            .count()
    })
}

fn char_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries: Vec<usize> = text.char_indices().map(|(idx, _)| idx).collect();
    boundaries.push(text.len());
    boundaries
}

fn byte_to_char_index(boundaries: &[usize], byte_index: usize) -> usize {
    match boundaries.binary_search(&byte_index) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
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

    fn projected_ids(notes: &[Note], query: &str, active_tag: Option<&str>) -> Vec<Uuid> {
        project_note_list(notes, None, query, active_tag)
            .rows
            .into_iter()
            .map(|row| row.id)
            .collect()
    }

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

        let projection = project_note_list(&notes, None, "rust", Some("work"));
        assert_eq!(
            projection.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![older_pinned.id, newer_unpinned.id]
        );
        let highlighted = &projection.rows[0].title_highlights;
        assert_eq!(
            highlighted.as_slice(),
            &[
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
    fn quoted_phrase_search_matches_multi_word_note_fragments() {
        let matching = Note::new(
            "Release planning".to_string(),
            "Capture migration risks before launch".to_string(),
        );
        let other = Note::new(
            "Migration".to_string(),
            "The launch notes live elsewhere".to_string(),
        );

        let ids = projected_ids(&[matching.clone(), other], "\"migration risks\"", None);

        assert_eq!(ids, vec![matching.id]);
    }

    #[test]
    fn plain_multi_word_search_preserves_substring_matching() {
        let matching = Note::new(
            "Release planning".to_string(),
            "Capture migration risks before launch".to_string(),
        );
        let words_apart = Note::new(
            "Migration".to_string(),
            "Risks are tracked before launch".to_string(),
        );

        let ids = projected_ids(&[matching.clone(), words_apart], "migration risks", None);

        assert_eq!(ids, vec![matching.id]);
    }

    #[test]
    fn malformed_quoted_phrase_search_fails_gently() {
        let matching = Note::new(
            "Release planning".to_string(),
            "Capture migration risks before launch".to_string(),
        );

        let ids = projected_ids(std::slice::from_ref(&matching), "\"migration risks", None);

        assert_eq!(ids, vec![matching.id]);
    }

    #[test]
    fn scoped_title_and_tag_search_combine_with_plain_search_and_active_tag() {
        let mut matching = Note::new(
            "Launch Plan".to_string(),
            "Review migration risks with product".to_string(),
        );
        matching.tags = vec!["Work".to_string(), "Product".to_string()];

        let mut missing_active_tag = Note::new(
            "Launch Plan".to_string(),
            "Review migration risks with product".to_string(),
        );
        missing_active_tag.tags = vec!["Work".to_string()];

        let mut missing_scoped_tag = Note::new(
            "Launch Plan".to_string(),
            "Review migration risks with product".to_string(),
        );
        missing_scoped_tag.tags = vec!["Product".to_string()];

        let notes = vec![
            matching.clone(),
            missing_active_tag,
            missing_scoped_tag,
            Note::new(
                "Migration Risks".to_string(),
                "Launch details only".to_string(),
            ),
        ];

        let projection = project_note_list(
            &notes,
            None,
            "title:launch tag:work \"migration risks\"",
            Some("product"),
        );

        assert_eq!(projection.rows.len(), 1);
        assert_eq!(projection.rows[0].id, matching.id);
        assert!(
            projection.rows[0]
                .title_highlights
                .iter()
                .any(|segment| segment.is_match && segment.text == "Launch")
        );
        assert!(
            projection.rows[0]
                .preview_highlights
                .iter()
                .any(|segment| segment.is_match && segment.text == "migration risks")
        );
    }

    #[test]
    fn scoped_search_sets_filtered_empty_state_when_no_notes_match() {
        let note = Note::new("Launch Plan".to_string(), "Review notes".to_string());

        let projection = project_note_list(&[note], None, "title:archive", None);

        assert!(projection.has_active_filter);
        assert!(projection.rows.is_empty());
    }

    #[test]
    fn projects_matched_tags_without_preview_highlights_for_scoped_tag_search() {
        let mut note = Note::new("Launch Plan".to_string(), "Review notes".to_string());
        note.tags = vec!["Work".to_string(), "Product".to_string()];

        let row = project_note_list(&[note], None, "tag:work", None)
            .rows
            .remove(0);

        assert_eq!(
            row.tag_highlights,
            vec![
                vec![HighlightSegment {
                    text: "Work".to_string(),
                    is_match: true,
                }],
                vec![HighlightSegment {
                    text: "Product".to_string(),
                    is_match: false,
                }],
            ]
        );
        assert_eq!(
            row.preview_highlights,
            vec![HighlightSegment {
                text: "Review notes".to_string(),
                is_match: false,
            }]
        );
    }

    #[test]
    fn body_match_uses_compact_match_snippet_when_title_and_tags_do_not_explain() {
        let mut note = Note::new(
            "Release Plan".to_string(),
            "Overview that does not explain the remembered phrase.\nLater review migration risks before launch with the team.".to_string(),
        );
        note.tags = vec!["Planning".to_string()];

        let row = project_note_list(&[note], None, "\"migration risks\"", None)
            .rows
            .remove(0);

        assert!(row.uses_match_snippet);
        assert!(row.preview.starts_with("..."));
        assert!(row.preview.contains("migration risks"));
        assert!(
            row.preview_highlights
                .iter()
                .any(|segment| segment.is_match && segment.text == "migration risks")
        );
    }

    #[test]
    fn title_explained_match_keeps_the_normal_preview() {
        let note = Note::new(
            "Migration Risks".to_string(),
            "Overview stays visible.\nLater migration risks appear in the body.".to_string(),
        );

        let row = project_note_list(&[note], None, "migration risks", None)
            .rows
            .remove(0);

        assert!(!row.uses_match_snippet);
        assert_eq!(row.preview, "Overview stays visible.");
    }

    #[test]
    fn body_match_snippet_prefers_the_window_that_explains_more_terms() {
        let mut note = Note::new(
            "Pinned Plan".to_string(),
            "Alpha appears alone near the opening. This spacer keeps beta out of reach. Later alpha and beta appear together near launch.".to_string(),
        );
        note.is_pinned = true;

        let row = project_note_list(&[note], None, "alpha is:pinned beta", None)
            .rows
            .remove(0);

        assert!(row.uses_match_snippet);
        assert!(row.preview.contains("alpha and beta"));
    }

    #[test]
    fn whitespace_search_uses_normal_note_list_ordering() {
        let mut older = Note::new("Older".to_string(), String::new());
        older.last_modified = Utc::now();
        let mut newer = Note::new("Newer".to_string(), String::new());
        newer.last_modified = older.last_modified + chrono::Duration::seconds(10);

        let projection = project_note_list(&[older.clone(), newer.clone()], None, "   ", None);

        assert!(!projection.has_active_filter);
        assert_eq!(
            projection.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![newer.id, older.id]
        );
    }

    #[test]
    fn pinned_search_filter_combines_with_plain_and_scoped_terms() {
        let mut older_pinned = Note::new(
            "Rust Launch Plan".to_string(),
            "Pinned migration risks".to_string(),
        );
        older_pinned.tags = vec!["Work".to_string()];
        older_pinned.is_pinned = true;
        older_pinned.last_modified = Utc::now();

        let mut newer_pinned = Note::new(
            "Rust Launch Plan".to_string(),
            "Pinned migration risks".to_string(),
        );
        newer_pinned.tags = vec!["Work".to_string()];
        newer_pinned.is_pinned = true;
        newer_pinned.last_modified = older_pinned.last_modified + chrono::Duration::seconds(10);

        let mut unpinned = Note::new(
            "Rust Launch Plan".to_string(),
            "Pinned migration risks".to_string(),
        );
        unpinned.tags = vec!["Work".to_string()];
        unpinned.last_modified = newer_pinned.last_modified + chrono::Duration::seconds(10);

        let ids = projected_ids(
            &[unpinned, older_pinned.clone(), newer_pinned.clone()],
            "is:pinned title:launch tag:work migration",
            None,
        );

        assert_eq!(ids, vec![newer_pinned.id, older_pinned.id]);
    }

    #[test]
    fn invalid_is_filter_does_not_block_plain_search_terms() {
        let matching = Note::new("Rust Launch Plan".to_string(), String::new());

        let ids = projected_ids(std::slice::from_ref(&matching), "is:archived rust", None);

        assert_eq!(ids, vec![matching.id]);
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
    fn reports_when_the_selected_note_is_hidden_by_the_active_filter() {
        let mut selected = Note::new("Writing draft".to_string(), "Private content".to_string());
        selected.tags = vec!["Personal".to_string()];
        let mut matching = Note::new("Launch plan".to_string(), "Work content".to_string());
        matching.tags = vec!["Work".to_string()];
        let notes = vec![selected.clone(), matching.clone()];

        let filtered = project_note_list(&notes, Some(selected.id), "", Some("work"));

        assert_eq!(
            filtered.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![matching.id]
        );
        assert_eq!(
            filtered.selected_note_visibility,
            SelectedNoteVisibility::HiddenByFilter
        );

        let unfiltered = project_note_list(&notes, Some(selected.id), "", None);

        assert!(unfiltered.rows.iter().any(|row| row.id == selected.id));
        assert_eq!(
            unfiltered.selected_note_visibility,
            SelectedNoteVisibility::Visible
        );
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
        let row = project_note_list(
            &[Note::new("İstanbul".to_string(), String::new())],
            None,
            "i",
            None,
        )
        .rows
        .remove(0);

        assert_eq!(
            row.title_highlights,
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
