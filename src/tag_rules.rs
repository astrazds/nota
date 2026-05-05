use std::collections::HashSet;

use crate::model::Note;

pub fn fold_case(text: &str) -> String {
    if text.is_ascii() {
        text.to_ascii_lowercase()
    } else {
        text.to_lowercase()
    }
}

pub fn parse_tags_input(input: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    input
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .filter_map(|tag| {
            let folded = fold_case(tag);
            if seen.insert(folded) {
                Some(tag.to_string())
            } else {
                None
            }
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
}
