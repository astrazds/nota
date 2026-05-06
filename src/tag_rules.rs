use std::collections::HashSet;

use crate::model::Note;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagSuggestion {
    pub label: String,
    pub completed_input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCleanupPlan {
    pub changes: Vec<TagCleanupChange>,
}

impl TagCleanupPlan {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCleanupChange {
    pub note_id: Uuid,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

pub fn fold_case(text: &str) -> String {
    if text.is_ascii() {
        text.to_ascii_lowercase()
    } else {
        text.to_lowercase()
    }
}

pub fn parse_tags_input(input: &str) -> Vec<String> {
    normalize_tags(input.split(',').map(str::to_string))
}

pub fn normalize_tags(tags: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    tags.into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .filter_map(|tag| {
            let folded = fold_case(&tag);
            if seen.insert(folded) { Some(tag) } else { None }
        })
        .collect()
}

pub fn normalize_tags_for_update(existing: &[String], tags: Vec<String>) -> Vec<String> {
    let mut canonical = Vec::new();
    let mut canonical_seen = HashSet::new();
    for tag in normalize_tags(existing.iter().cloned()) {
        let folded = fold_case(&tag);
        if canonical_seen.insert(folded) {
            canonical.push(tag);
        }
    }

    let mut seen = HashSet::new();
    tags.into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .filter_map(|tag| {
            let folded = fold_case(&tag);
            if !seen.insert(folded.clone()) {
                return None;
            }

            Some(
                canonical
                    .iter()
                    .find(|existing| tags_equal(existing, &tag))
                    .cloned()
                    .unwrap_or(tag),
            )
        })
        .collect()
}

pub fn tags_to_input(tags: &[String]) -> String {
    tags.join(", ")
}

pub fn tags_equal(left: &str, right: &str) -> bool {
    fold_case(left.trim()) == fold_case(right.trim())
}

pub fn note_has_active_tag(note: &Note, active_tag: &str) -> bool {
    note.tags.iter().any(|tag| tags_equal(tag, active_tag))
}

pub fn collect_note_tags(notes: &[Note]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tags = Vec::new();

    for note in notes {
        for tag in &note.tags {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                continue;
            }

            let folded = fold_case(trimmed);
            if seen.insert(folded) {
                tags.push(trimmed.to_string());
            }
        }
    }

    tags.sort_by(|a, b| fold_case(a).cmp(&fold_case(b)).then_with(|| a.cmp(b)));
    tags
}

pub fn suggest_existing_tags(
    notes: &[Note],
    selected_note: Option<&Note>,
    input: &str,
) -> Vec<TagSuggestion> {
    let excluded = selected_note
        .map(|note| {
            note.tags
                .iter()
                .map(|tag| fold_case(tag.trim()))
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let fragment = current_tag_fragment(input);
    let folded_fragment = fold_case(fragment);

    collect_note_tags(notes)
        .into_iter()
        .filter(|tag| !excluded.contains(&fold_case(tag)))
        .filter(|tag| folded_fragment.is_empty() || fold_case(tag).starts_with(&folded_fragment))
        .map(|tag| TagSuggestion {
            completed_input: completed_input_with_tag(input, &tag),
            label: tag,
        })
        .collect()
}

pub fn remove_tag(tags: &[String], tag_to_remove: &str) -> Vec<String> {
    normalize_tags(
        tags.iter()
            .filter(|tag| !tags_equal(tag, tag_to_remove))
            .cloned(),
    )
}

pub fn plan_collection_tag_cleanup(notes: &[Note]) -> TagCleanupPlan {
    let canonical = collection_canonical_tags(notes);
    let changes = notes
        .iter()
        .filter_map(|note| {
            let after = normalize_with_canonical(&note.tags, &canonical);
            (note.tags != after).then(|| TagCleanupChange {
                note_id: note.id,
                before: note.tags.clone(),
                after,
            })
        })
        .collect();

    TagCleanupPlan { changes }
}

pub fn apply_tag_cleanup_plan(notes: &mut [Note], plan: &TagCleanupPlan) -> Vec<Uuid> {
    let mut changed = Vec::new();

    for change in &plan.changes {
        let Some(note) = notes.iter_mut().find(|note| note.id == change.note_id) else {
            continue;
        };

        if note.tags != change.before || note.tags == change.after {
            continue;
        }

        note.tags.clone_from(&change.after);
        changed.push(note.id);
    }

    changed
}

fn current_tag_fragment(input: &str) -> &str {
    input
        .rsplit_once(',')
        .map_or(input, |(_, fragment)| fragment)
        .trim()
}

fn completed_input_with_tag(input: &str, tag: &str) -> String {
    let prefix = input
        .rsplit_once(',')
        .map(|(prefix, _)| parse_tags_input(prefix))
        .unwrap_or_default();
    let mut tags = prefix;
    tags.push(tag.to_string());
    tags_to_input(&normalize_tags(tags))
}

fn collection_canonical_tags(notes: &[Note]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut canonical = Vec::new();

    for note in notes {
        for tag in &note.tags {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                continue;
            }

            let folded = fold_case(trimmed);
            if seen.insert(folded) {
                canonical.push(trimmed.to_string());
            }
        }
    }

    canonical
}

fn normalize_with_canonical(tags: &[String], canonical: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    tags.iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .filter_map(|tag| {
            let folded = fold_case(&tag);
            if !seen.insert(folded) {
                return None;
            }

            Some(
                canonical
                    .iter()
                    .find(|canonical_tag| tags_equal(canonical_tag, &tag))
                    .cloned()
                    .unwrap_or(tag),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_display_tags_from_comma_separated_input() {
        let tags = parse_tags_input(" work, rust,work, , Product ");

        assert_eq!(
            tags,
            vec![
                "work".to_string(),
                "rust".to_string(),
                "Product".to_string()
            ]
        );
        assert_eq!(tags_to_input(&tags), "work, rust, Product");
    }

    #[test]
    fn matches_tags_case_insensitively() {
        assert!(tags_equal(" Work ", "work"));
        assert!(fold_case("Project").contains(&fold_case("pro")));
        assert!(!tags_equal("Work", "Personal"));
    }

    #[test]
    fn collects_unique_sorted_tags_from_notes() {
        let mut first = Note::new("One".to_string(), String::new());
        first.tags = vec!["Rust".to_string(), "work".to_string()];
        let mut second = Note::new("Two".to_string(), String::new());
        second.tags = vec!["rust".to_string(), "Personal".to_string(), " ".to_string()];

        assert_eq!(
            collect_note_tags(&[first, second]),
            vec![
                "Personal".to_string(),
                "Rust".to_string(),
                "work".to_string()
            ]
        );
    }

    #[test]
    fn suggests_collection_tags_for_the_current_entry_without_blocking_free_form_tags() {
        let mut selected = Note::new("Selected".to_string(), String::new());
        selected.tags = vec!["Work".to_string()];
        let mut other = Note::new("Other".to_string(), String::new());
        other.tags = vec![
            "Research".to_string(),
            "rust".to_string(),
            "Work".to_string(),
            "Roadmap".to_string(),
        ];

        let suggestions =
            suggest_existing_tags(&[selected.clone(), other], Some(&selected), "Plan, r");

        assert_eq!(
            suggestions,
            vec![
                TagSuggestion {
                    label: "Research".to_string(),
                    completed_input: "Plan, Research".to_string(),
                },
                TagSuggestion {
                    label: "Roadmap".to_string(),
                    completed_input: "Plan, Roadmap".to_string(),
                },
                TagSuggestion {
                    label: "rust".to_string(),
                    completed_input: "Plan, rust".to_string(),
                },
            ]
        );
    }

    #[test]
    fn plans_collection_tag_cleanup_without_mutating_notes() {
        let mut first = Note::new("First".to_string(), String::new());
        first.tags = vec![
            " Rust ".to_string(),
            "rust".to_string(),
            "".to_string(),
            "Work".to_string(),
        ];
        let mut second = Note::new("Second".to_string(), String::new());
        second.tags = vec![
            "rust".to_string(),
            " personal ".to_string(),
            "PERSONAL".to_string(),
        ];
        let mut clean = Note::new("Clean".to_string(), String::new());
        clean.tags = vec!["Reference".to_string()];
        let notes = vec![first.clone(), second.clone(), clean];

        let plan = plan_collection_tag_cleanup(&notes);

        assert_eq!(
            plan.changes,
            vec![
                TagCleanupChange {
                    note_id: first.id,
                    before: vec![
                        " Rust ".to_string(),
                        "rust".to_string(),
                        "".to_string(),
                        "Work".to_string(),
                    ],
                    after: vec!["Rust".to_string(), "Work".to_string()],
                },
                TagCleanupChange {
                    note_id: second.id,
                    before: vec![
                        "rust".to_string(),
                        " personal ".to_string(),
                        "PERSONAL".to_string(),
                    ],
                    after: vec!["Rust".to_string(), "personal".to_string()],
                },
            ]
        );
        assert_eq!(notes[0].tags, first.tags);
        assert_eq!(notes[1].tags, second.tags);
    }

    #[test]
    fn cleanup_plan_apply_is_conservative_about_changed_notes() {
        let mut first = Note::new("First".to_string(), String::new());
        first.tags = vec![" Work ".to_string(), "work".to_string()];
        let mut second = Note::new("Second".to_string(), String::new());
        second.tags = vec!["Work".to_string()];
        let mut notes = vec![first.clone(), second.clone()];
        let plan = plan_collection_tag_cleanup(&notes);

        notes[0].tags = vec!["User changed".to_string()];

        assert!(!plan.is_empty());
        assert!(apply_tag_cleanup_plan(&mut notes, &plan).is_empty());
        assert_eq!(notes[0].tags, vec!["User changed".to_string()]);
        assert_eq!(notes[1].tags, vec!["Work".to_string()]);
    }
}
