use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Note;
use crate::backup::BackupHealthRecord;

const TRANSITION_VERSION: u32 = 1;
const TRANSITION_KIND: &str = "noter.desktop_transition";

pub fn desktop_transition_file_name(now: DateTime<Utc>) -> String {
    format!("noter-desktop-transition-{}.json", now.format("%Y-%m-%d"))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesktopTransitionBundle {
    pub version: u32,
    pub kind: String,
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
    pub theme: ThemePreference,
    pub backup_health: Option<BackupHealthRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRestore {
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
    pub theme: ThemePreference,
    pub backup_health: Option<BackupHealthRecord>,
}

#[derive(Debug)]
pub enum TransitionError {
    Deserialize(serde_json::Error),
    Serialize(serde_json::Error),
    UnsupportedVersion(u32),
    UnsupportedKind(String),
    DuplicateNoteId(Uuid),
    CollectionNotEmpty,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialize(error) => {
                write!(formatter, "invalid desktop transition JSON: {error}")
            }
            Self::Serialize(error) => {
                write!(formatter, "could not serialize desktop transition: {error}")
            }
            Self::UnsupportedVersion(version) => write!(
                formatter,
                "unsupported desktop transition version {version}"
            ),
            Self::UnsupportedKind(kind) => {
                write!(formatter, "unsupported desktop transition kind {kind}")
            }
            Self::DuplicateNoteId(id) => write!(formatter, "duplicate Note identity {id}"),
            Self::CollectionNotEmpty => write!(
                formatter,
                "desktop transition restore requires an Empty Collection; use merge Backup import instead"
            ),
        }
    }
}

impl Error for TransitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Deserialize(error) | Self::Serialize(error) => Some(error),
            Self::UnsupportedVersion(_)
            | Self::UnsupportedKind(_)
            | Self::DuplicateNoteId(_)
            | Self::CollectionNotEmpty => None,
        }
    }
}

pub fn export_desktop_transition(
    notes: &[Note],
    recently_deleted_notes: &[Note],
    theme: ThemePreference,
    backup_health: Option<BackupHealthRecord>,
) -> Result<String, TransitionError> {
    validate_unique_ids(notes, recently_deleted_notes)?;
    serde_json::to_string_pretty(&DesktopTransitionBundle {
        version: TRANSITION_VERSION,
        kind: TRANSITION_KIND.to_string(),
        notes: notes.to_vec(),
        recently_deleted_notes: recently_deleted_notes.to_vec(),
        theme,
        backup_health,
    })
    .map_err(TransitionError::Serialize)
}

pub fn import_desktop_transition(
    current_notes: &[Note],
    current_recently_deleted_notes: &[Note],
    transition_json: &str,
) -> Result<TransitionRestore, TransitionError> {
    if !current_notes.is_empty() || !current_recently_deleted_notes.is_empty() {
        return Err(TransitionError::CollectionNotEmpty);
    }

    let bundle: DesktopTransitionBundle =
        serde_json::from_str(transition_json).map_err(TransitionError::Deserialize)?;
    if bundle.version != TRANSITION_VERSION {
        return Err(TransitionError::UnsupportedVersion(bundle.version));
    }
    if bundle.kind != TRANSITION_KIND {
        return Err(TransitionError::UnsupportedKind(bundle.kind));
    }
    validate_unique_ids(&bundle.notes, &bundle.recently_deleted_notes)?;

    Ok(TransitionRestore {
        notes: bundle.notes,
        recently_deleted_notes: bundle.recently_deleted_notes,
        theme: bundle.theme,
        backup_health: bundle.backup_health,
    })
}

fn validate_unique_ids(
    notes: &[Note],
    recently_deleted_notes: &[Note],
) -> Result<(), TransitionError> {
    let mut seen = HashSet::new();
    for note in notes.iter().chain(recently_deleted_notes) {
        if !seen.insert(note.id) {
            return Err(TransitionError::DuplicateNoteId(note.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn exact_restore_is_allowed_only_into_an_empty_collection() {
        let active = Note::new("Active".to_string(), "Body".to_string());
        let deleted = Note::new("Deleted".to_string(), "Recover me".to_string());
        let json = export_desktop_transition(
            std::slice::from_ref(&active),
            std::slice::from_ref(&deleted),
            ThemePreference::Dark,
            None,
        )
        .unwrap();

        let restored = import_desktop_transition(&[], &[], &json).unwrap();
        assert_eq!(restored.notes, vec![active.clone()]);
        assert_eq!(restored.recently_deleted_notes, vec![deleted]);
        assert_eq!(restored.theme, ThemePreference::Dark);

        let error = import_desktop_transition(std::slice::from_ref(&active), &[], &json)
            .expect_err("a non-empty native collection must reject exact restore");
        assert!(matches!(error, TransitionError::CollectionNotEmpty));
    }

    #[test]
    fn duplicate_identities_across_active_and_deleted_are_rejected() {
        let note = Note::new("Duplicate".to_string(), String::new());
        let error = export_desktop_transition(
            std::slice::from_ref(&note),
            std::slice::from_ref(&note),
            ThemePreference::System,
            None,
        )
        .expect_err("one Note identity cannot exist in both collections");

        assert!(matches!(error, TransitionError::DuplicateNoteId(id) if id == note.id));
    }

    #[test]
    fn transition_filename_is_distinct_from_normal_backup() {
        let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
        assert_eq!(
            desktop_transition_file_name(now),
            "noter-desktop-transition-2026-09-03.json"
        );
    }
}
