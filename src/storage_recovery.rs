#![cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]

use crate::model::Note;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCollectionPayload {
    pub notes_json: Option<String>,
    pub recently_deleted_notes_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviousSnapshot {
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageRecoveryState {
    pub corrupt_notes_json: Option<String>,
    pub corrupt_recently_deleted_notes_json: Option<String>,
    pub previous_snapshot: Option<PreviousSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionStartup {
    Ready {
        notes: Vec<Note>,
        recently_deleted_notes: Vec<Note>,
    },
    Recovery(StorageRecoveryState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSavePlan {
    pub next_notes_json: String,
    pub next_recently_deleted_notes_json: String,
    pub previous_notes_json: Option<String>,
    pub previous_recently_deleted_notes_json: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageRecoveryError {
    InvalidNextNotes,
    InvalidNextRecentlyDeletedNotes,
}

pub fn decide_collection_startup(
    current: StoredCollectionPayload,
    previous: StoredCollectionPayload,
    starter_notes: Vec<Note>,
) -> CollectionStartup {
    let parsed_notes = parse_optional_notes(current.notes_json.as_deref());
    let parsed_recently_deleted =
        parse_optional_notes(current.recently_deleted_notes_json.as_deref());

    if parsed_notes.is_err() || parsed_recently_deleted.is_err() {
        return CollectionStartup::Recovery(StorageRecoveryState {
            corrupt_notes_json: parsed_notes.err().and(current.notes_json),
            corrupt_recently_deleted_notes_json: parsed_recently_deleted
                .err()
                .and(current.recently_deleted_notes_json),
            previous_snapshot: previous_snapshot(previous),
        });
    }

    let notes = match parsed_notes.unwrap_or_default() {
        Some(notes) => notes,
        None if starter_notes.is_empty() => Vec::new(),
        None => starter_notes,
    };
    let recently_deleted_notes = parsed_recently_deleted
        .unwrap_or_default()
        .unwrap_or_default();

    CollectionStartup::Ready {
        notes,
        recently_deleted_notes,
    }
}

pub fn plan_collection_save_from_json(
    current: StoredCollectionPayload,
    next_notes_json: String,
    next_recently_deleted_notes_json: String,
) -> Result<CollectionSavePlan, StorageRecoveryError> {
    parse_notes(&next_notes_json).map_err(|_| StorageRecoveryError::InvalidNextNotes)?;
    parse_notes(&next_recently_deleted_notes_json)
        .map_err(|_| StorageRecoveryError::InvalidNextRecentlyDeletedNotes)?;

    let previous_notes_json = current.notes_json.filter(|json| parse_notes(json).is_ok());
    let previous_recently_deleted_notes_json = current
        .recently_deleted_notes_json
        .filter(|json| parse_notes(json).is_ok())
        .or_else(|| previous_notes_json.as_ref().map(|_| "[]".to_string()));

    Ok(CollectionSavePlan {
        next_notes_json,
        next_recently_deleted_notes_json,
        previous_notes_json,
        previous_recently_deleted_notes_json,
    })
}

fn previous_snapshot(previous: StoredCollectionPayload) -> Option<PreviousSnapshot> {
    let notes_json = previous.notes_json?;
    let recently_deleted_notes_json = previous.recently_deleted_notes_json?;
    Some(PreviousSnapshot {
        notes: parse_notes(&notes_json).ok()?,
        recently_deleted_notes: parse_notes(&recently_deleted_notes_json).ok()?,
    })
}

fn parse_optional_notes(raw: Option<&str>) -> Result<Option<Vec<Note>>, serde_json::Error> {
    raw.map(parse_notes).transpose()
}

fn parse_notes(raw: &str) -> Result<Vec<Note>, serde_json::Error> {
    serde_json::from_str::<Vec<Note>>(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes_json(notes: &[Note]) -> String {
        serde_json::to_string(notes).unwrap()
    }

    #[test]
    fn safe_save_preserves_the_current_collection_pair_after_next_payload_validates() {
        let active = Note::new("Active".to_string(), "Current".to_string());
        let deleted = Note::new("Deleted".to_string(), "Recover".to_string());
        let next = Note::new("Next".to_string(), "Future".to_string());
        let current_notes_json = notes_json(std::slice::from_ref(&active));
        let current_deleted_json = notes_json(std::slice::from_ref(&deleted));
        let next_notes_json = notes_json(std::slice::from_ref(&next));
        let next_deleted_json = "[]".to_string();

        let plan = plan_collection_save_from_json(
            StoredCollectionPayload {
                notes_json: Some(current_notes_json.clone()),
                recently_deleted_notes_json: Some(current_deleted_json.clone()),
            },
            next_notes_json.clone(),
            next_deleted_json.clone(),
        )
        .unwrap();

        assert_eq!(plan.next_notes_json, next_notes_json);
        assert_eq!(plan.next_recently_deleted_notes_json, next_deleted_json);
        assert_eq!(plan.previous_notes_json, Some(current_notes_json));
        assert_eq!(
            plan.previous_recently_deleted_notes_json,
            Some(current_deleted_json)
        );
    }

    #[test]
    fn invalid_next_payload_does_not_create_a_save_plan() {
        let current = Note::new("Current".to_string(), "Safe".to_string());
        let current_notes_json = notes_json(&[current]);

        let result = plan_collection_save_from_json(
            StoredCollectionPayload {
                notes_json: Some(current_notes_json),
                recently_deleted_notes_json: Some("[]".to_string()),
            },
            "{not valid json".to_string(),
            "[]".to_string(),
        );

        assert_eq!(result, Err(StorageRecoveryError::InvalidNextNotes));
    }

    #[test]
    fn corrupt_startup_enters_recovery_with_previous_snapshot_available() {
        let previous_active = Note::new("Previous".to_string(), "Known good".to_string());
        let previous_deleted = Note::new("Previous deleted".to_string(), "Recover".to_string());
        let startup = decide_collection_startup(
            StoredCollectionPayload {
                notes_json: Some("{not valid json".to_string()),
                recently_deleted_notes_json: Some("[]".to_string()),
            },
            StoredCollectionPayload {
                notes_json: Some(notes_json(std::slice::from_ref(&previous_active))),
                recently_deleted_notes_json: Some(notes_json(std::slice::from_ref(
                    &previous_deleted,
                ))),
            },
            Vec::new(),
        );

        assert_eq!(
            startup,
            CollectionStartup::Recovery(StorageRecoveryState {
                corrupt_notes_json: Some("{not valid json".to_string()),
                corrupt_recently_deleted_notes_json: None,
                previous_snapshot: Some(PreviousSnapshot {
                    notes: vec![previous_active],
                    recently_deleted_notes: vec![previous_deleted],
                }),
            })
        );
    }
}
