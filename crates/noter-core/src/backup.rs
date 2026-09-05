use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Note;

const BACKUP_VERSION: u32 = 1;
const BACKUP_KIND: &str = "noter.flat_collection";
const BACKUP_HEALTH_STALE_AFTER_DAYS: i64 = 14;

#[derive(Debug, Serialize, Deserialize)]
struct FlatCollectionBackup {
    version: u32,
    kind: String,
    notes: Vec<Note>,
}

#[derive(Debug)]
pub enum BackupError {
    Deserialize(serde_json::Error),
    Serialize(serde_json::Error),
    UnsupportedVersion(u32),
    UnsupportedKind(String),
    DuplicateNoteId(Uuid),
}

impl fmt::Display for BackupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialize(error) => write!(formatter, "invalid Backup JSON: {error}"),
            Self::Serialize(error) => write!(formatter, "could not serialize Backup: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported Backup version {version}")
            }
            Self::UnsupportedKind(kind) => write!(formatter, "unsupported Backup kind {kind}"),
            Self::DuplicateNoteId(id) => write!(formatter, "duplicate Note identity {id}"),
        }
    }
}

impl Error for BackupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Deserialize(error) | Self::Serialize(error) => Some(error),
            Self::UnsupportedVersion(_) | Self::UnsupportedKind(_) | Self::DuplicateNoteId(_) => {
                None
            }
        }
    }
}

pub fn export_flat_collection_backup(notes: &[Note]) -> Result<String, BackupError> {
    serde_json::to_string_pretty(&FlatCollectionBackup {
        version: BACKUP_VERSION,
        kind: BACKUP_KIND.to_string(),
        notes: notes.to_vec(),
    })
    .map_err(BackupError::Serialize)
}

pub fn backup_file_name(now: DateTime<Utc>) -> String {
    format!("nota-backup-{}.json", now.format("%Y-%m-%d"))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupHealthRecord {
    pub last_successful_export_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupHealth {
    Missing,
    Recent {
        last_successful_export_at: DateTime<Utc>,
    },
    Stale {
        last_successful_export_at: DateTime<Utc>,
    },
}

pub fn assess_backup_health(
    record: Option<BackupHealthRecord>,
    now: DateTime<Utc>,
) -> BackupHealth {
    let Some(record) = record else {
        return BackupHealth::Missing;
    };
    if now.signed_duration_since(record.last_successful_export_at)
        > chrono::Duration::days(BACKUP_HEALTH_STALE_AFTER_DAYS)
    {
        BackupHealth::Stale {
            last_successful_export_at: record.last_successful_export_at,
        }
    } else {
        BackupHealth::Recent {
            last_successful_export_at: record.last_successful_export_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupImport {
    pub selected_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupImportPreview {
    pub notes_to_add: usize,
    pub notes_to_replace: usize,
    pub total_imported_notes: usize,
    pub selected_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingBackupImport {
    pub backup_json: String,
    pub preview: BackupImportPreview,
}

pub fn prepare_backup_import(
    notes: &[Note],
    backup_json: String,
) -> Result<PendingBackupImport, BackupError> {
    let preview = preview_flat_collection_backup(notes, &backup_json)?;
    Ok(PendingBackupImport {
        backup_json,
        preview,
    })
}

pub fn preview_flat_collection_backup(
    notes: &[Note],
    backup_json: &str,
) -> Result<BackupImportPreview, BackupError> {
    let backup = parse_flat_collection_backup(backup_json)?;
    Ok(import_preview(notes, &backup.notes))
}

pub fn import_flat_collection_backup(
    notes: &mut Vec<Note>,
    backup_json: &str,
) -> Result<BackupImport, BackupError> {
    let backup = parse_flat_collection_backup(backup_json)?;
    let selected_id = backup.notes.first().map(|note| note.id);
    merge_notes(notes, backup.notes);
    Ok(BackupImport { selected_id })
}

fn parse_flat_collection_backup(backup_json: &str) -> Result<FlatCollectionBackup, BackupError> {
    let backup = serde_json::from_str(backup_json).map_err(BackupError::Deserialize)?;
    validate_backup(&backup)?;
    Ok(backup)
}

fn import_preview(notes: &[Note], backup_notes: &[Note]) -> BackupImportPreview {
    let notes_to_replace = backup_notes
        .iter()
        .filter(|backup_note| notes.iter().any(|note| note.id == backup_note.id))
        .count();
    BackupImportPreview {
        notes_to_add: backup_notes.len().saturating_sub(notes_to_replace),
        notes_to_replace,
        total_imported_notes: backup_notes.len(),
        selected_id: backup_notes.first().map(|note| note.id),
    }
}

fn merge_notes(notes: &mut Vec<Note>, backup_notes: Vec<Note>) {
    for backup_note in backup_notes {
        if let Some(existing_note) = notes.iter_mut().find(|note| note.id == backup_note.id) {
            *existing_note = backup_note;
        } else {
            notes.push(backup_note);
        }
    }
}

fn validate_backup(backup: &FlatCollectionBackup) -> Result<(), BackupError> {
    if backup.version != BACKUP_VERSION {
        return Err(BackupError::UnsupportedVersion(backup.version));
    }
    if backup.kind != BACKUP_KIND {
        return Err(BackupError::UnsupportedKind(backup.kind.clone()));
    }
    let mut seen_ids = HashSet::new();
    for note in &backup.notes {
        if !seen_ids.insert(note.id) {
            return Err(BackupError::DuplicateNoteId(note.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_v1_round_trips_through_the_public_interface() {
        let note = Note::new("Release notes".to_string(), "Ship native".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&note)).unwrap();
        let mut restored = Vec::new();

        import_flat_collection_backup(&mut restored, &json).unwrap();

        assert_eq!(restored, vec![note]);
    }
}
