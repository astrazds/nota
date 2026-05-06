use crate::model::Note;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    DuplicateNoteId(uuid::Uuid),
}

pub fn export_flat_collection_backup(notes: &[Note]) -> Result<String, BackupError> {
    let backup = FlatCollectionBackup {
        version: BACKUP_VERSION,
        kind: BACKUP_KIND.to_string(),
        notes: notes.to_vec(),
    };

    serde_json::to_string_pretty(&backup).map_err(BackupError::Serialize)
}

pub fn backup_file_name(now: DateTime<Utc>) -> String {
    format!("noter-backup-{}.json", now.format("%Y-%m-%d"))
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
    pub selected_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupImportPreview {
    pub notes_to_add: usize,
    pub notes_to_replace: usize,
    pub total_imported_notes: usize,
    pub selected_id: Option<uuid::Uuid>,
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
    let backup: FlatCollectionBackup =
        serde_json::from_str(backup_json).map_err(BackupError::Deserialize)?;
    validate_backup(&backup)?;
    Ok(backup)
}

fn import_preview(notes: &[Note], backup_notes: &[Note]) -> BackupImportPreview {
    let notes_to_replace = backup_notes
        .iter()
        .filter(|backup_note| notes.iter().any(|note| note.id == backup_note.id))
        .count();
    let total_imported_notes = backup_notes.len();
    BackupImportPreview {
        notes_to_add: total_imported_notes.saturating_sub(notes_to_replace),
        notes_to_replace,
        total_imported_notes,
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

    let mut seen_ids = std::collections::HashSet::new();
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
    use crate::model::Note;
    use chrono::{TimeZone, Utc};
    use serde_json::Value;
    use uuid::Uuid;

    fn note_with_fields() -> Note {
        Note {
            id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            title: "Release notes".to_string(),
            content: "# Shipped\n\n- Backup export".to_string(),
            tags: vec!["work".to_string(), "release".to_string()],
            is_pinned: true,
            created: Utc.with_ymd_and_hms(2026, 4, 1, 9, 30, 0).unwrap(),
            last_modified: Utc.with_ymd_and_hms(2026, 4, 2, 10, 45, 0).unwrap(),
        }
    }

    #[test]
    fn exports_a_versioned_human_inspectable_flat_collection_backup() {
        let backup_json = export_flat_collection_backup(&[note_with_fields()]).unwrap();
        let backup: Value = serde_json::from_str(&backup_json).unwrap();

        assert_eq!(backup["version"], 1);
        assert_eq!(backup["kind"], "noter.flat_collection");
        assert_eq!(
            backup["notes"][0]["id"],
            "11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(backup["notes"][0]["title"], "Release notes");
        assert_eq!(
            backup["notes"][0]["content"],
            "# Shipped\n\n- Backup export"
        );
        assert_eq!(
            backup["notes"][0]["tags"],
            serde_json::json!(["work", "release"])
        );
        assert_eq!(backup["notes"][0]["is_pinned"], true);
        assert_eq!(backup["notes"][0]["created"], "2026-04-01T09:30:00Z");
        assert_eq!(backup["notes"][0]["last_modified"], "2026-04-02T10:45:00Z");
    }

    #[test]
    fn exports_an_empty_flat_collection_backup() {
        let backup_json = export_flat_collection_backup(&[]).unwrap();
        let backup: Value = serde_json::from_str(&backup_json).unwrap();

        assert_eq!(backup["version"], 1);
        assert_eq!(backup["kind"], "noter.flat_collection");
        assert_eq!(backup["notes"], serde_json::json!([]));
    }

    #[test]
    fn backup_filename_identifies_noter_and_the_current_date() {
        let now = Utc.with_ymd_and_hms(2026, 5, 6, 9, 30, 0).unwrap();

        assert_eq!(backup_file_name(now), "noter-backup-2026-05-06.json");
    }

    #[test]
    fn backup_health_distinguishes_missing_recent_and_stale_exports() {
        let now = Utc.with_ymd_and_hms(2026, 5, 6, 9, 30, 0).unwrap();

        assert_eq!(assess_backup_health(None, now), BackupHealth::Missing);

        let recent_record = BackupHealthRecord {
            last_successful_export_at: now - chrono::Duration::days(3),
        };
        assert_eq!(
            assess_backup_health(Some(recent_record), now),
            BackupHealth::Recent {
                last_successful_export_at: recent_record.last_successful_export_at
            }
        );

        let stale_record = BackupHealthRecord {
            last_successful_export_at: now - chrono::Duration::days(30),
        };
        assert_eq!(
            assess_backup_health(Some(stale_record), now),
            BackupHealth::Stale {
                last_successful_export_at: stale_record.last_successful_export_at
            }
        );
    }

    #[test]
    fn imports_into_an_empty_collection_and_preserves_note_fields() {
        let original_note = note_with_fields();
        let backup_json =
            export_flat_collection_backup(std::slice::from_ref(&original_note)).unwrap();
        let mut notes = Vec::new();

        let imported = import_flat_collection_backup(&mut notes, &backup_json).unwrap();

        assert_eq!(imported.selected_id, Some(original_note.id));
        assert_eq!(notes, vec![original_note]);
    }

    #[test]
    fn merge_import_replaces_same_identity_notes_without_creating_duplicates() {
        let original_note = note_with_fields();
        let mut replacement_note = original_note.clone();
        replacement_note.title = "Imported replacement".to_string();
        replacement_note.content = "Backup content wins".to_string();

        let existing_only = Note {
            id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
            title: "Keep me".to_string(),
            content: "Existing content".to_string(),
            tags: Vec::new(),
            is_pinned: false,
            created: Utc.with_ymd_and_hms(2026, 3, 1, 9, 0, 0).unwrap(),
            last_modified: Utc.with_ymd_and_hms(2026, 3, 2, 9, 0, 0).unwrap(),
        };
        let imported_only = Note {
            id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
            title: "Imported only".to_string(),
            content: "New imported content".to_string(),
            tags: vec!["new".to_string()],
            is_pinned: false,
            created: Utc.with_ymd_and_hms(2026, 5, 1, 9, 0, 0).unwrap(),
            last_modified: Utc.with_ymd_and_hms(2026, 5, 2, 9, 0, 0).unwrap(),
        };
        let backup_json =
            export_flat_collection_backup(&[replacement_note.clone(), imported_only.clone()])
                .unwrap();
        let mut notes = vec![original_note, existing_only.clone()];

        let imported = import_flat_collection_backup(&mut notes, &backup_json).unwrap();

        assert_eq!(imported.selected_id, Some(replacement_note.id));
        assert_eq!(notes, vec![replacement_note, existing_only, imported_only]);
    }

    #[test]
    fn previews_merge_import_impact_without_mutating_current_notes() {
        let original_note = note_with_fields();
        let mut replacement_note = original_note.clone();
        replacement_note.title = "Imported replacement".to_string();
        let imported_only = Note {
            id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
            title: "Imported only".to_string(),
            content: "New imported content".to_string(),
            tags: vec!["new".to_string()],
            is_pinned: false,
            created: Utc.with_ymd_and_hms(2026, 5, 1, 9, 0, 0).unwrap(),
            last_modified: Utc.with_ymd_and_hms(2026, 5, 2, 9, 0, 0).unwrap(),
        };
        let backup_json =
            export_flat_collection_backup(&[replacement_note.clone(), imported_only.clone()])
                .unwrap();
        let notes = vec![original_note.clone()];

        let preview = preview_flat_collection_backup(&notes, &backup_json).unwrap();

        assert_eq!(preview.notes_to_add, 1);
        assert_eq!(preview.notes_to_replace, 1);
        assert_eq!(preview.total_imported_notes, 2);
        assert_eq!(preview.selected_id, Some(replacement_note.id));
        assert_eq!(notes, vec![original_note]);
    }

    #[test]
    fn invalid_backups_fail_without_mutating_the_current_collection() {
        let original_notes = vec![note_with_fields()];
        let invalid_cases = [
            "{not valid json".to_string(),
            serde_json::json!({
                "version": 2,
                "kind": "noter.flat_collection",
                "notes": []
            })
            .to_string(),
            serde_json::json!({
                "version": 1,
                "kind": "noter.flat_collection",
                "notes": [{
                    "id": "not-a-uuid",
                    "title": "Broken",
                    "content": "Cannot import",
                    "tags": [],
                    "is_pinned": false,
                    "created": "2026-04-01T09:30:00Z",
                    "last_modified": "2026-04-02T10:45:00Z"
                }]
            })
            .to_string(),
            serde_json::json!({
                "version": 1,
                "kind": "noter.flat_collection",
                "notes": [
                    note_with_fields(),
                    note_with_fields()
                ]
            })
            .to_string(),
        ];

        for invalid_backup in invalid_cases {
            let mut notes = original_notes.clone();

            let result = import_flat_collection_backup(&mut notes, &invalid_backup);

            assert!(result.is_err());
            assert_eq!(notes, original_notes);
        }
    }
}
