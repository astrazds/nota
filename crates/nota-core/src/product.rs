use serde::Serialize;

/// Sentences a shell paints. The shell does not own the wording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductText {
    pub title: String,
    pub body: String,
}

/// The one confirmation the session is waiting on, plus the same shape for tag cleanup copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Prompt {
    pub title: String,
    pub body: String,
    pub action: String,
}

pub fn delete_prompt(title: &str) -> Prompt {
    Prompt {
        title: "Move to Recently Deleted?".to_string(),
        body: format!("“{title}” will move to Recently Deleted."),
        action: "Move".to_string(),
    }
}

pub fn permanent_delete_prompt() -> Prompt {
    Prompt {
        title: "Delete this note permanently?".to_string(),
        body: "This cannot be undone. Use Backup / Export in the Backup menu first if you want to keep a copy.".to_string(),
        action: "Delete".to_string(),
    }
}

pub fn clear_deleted_prompt(count: usize) -> Prompt {
    let note_label = if count == 1 { "Note" } else { "Notes" };
    Prompt {
        title: "Permanently clear Recently Deleted?".to_string(),
        body: format!("This will permanently clear {count} recently deleted {note_label}."),
        action: "Clear All".to_string(),
    }
}

pub fn tag_cleanup_prompt(detail: String) -> Prompt {
    Prompt {
        title: "Clean up Tags?".to_string(),
        body: detail,
        action: "Apply cleanup".to_string(),
    }
}
