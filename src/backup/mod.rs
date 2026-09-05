pub(crate) mod controls;
pub use nota_core::backup::*;

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
        assert_eq!(backup["kind"], "nota.flat_collection");
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
        assert_eq!(backup["kind"], "nota.flat_collection");
        assert_eq!(backup["notes"], serde_json::json!([]));
    }

    #[test]
    fn backup_filename_identifies_nota_and_the_current_date() {
        let now = Utc.with_ymd_and_hms(2026, 5, 6, 9, 30, 0).unwrap();

        assert_eq!(backup_file_name(now), "nota-backup-2026-05-06.json");
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

    #[test]
    fn prepares_a_pending_backup_import_with_preview_counts() {
        let existing = note_with_fields();
        let mut replacement = existing.clone();
        replacement.content = "Backup content wins".to_string();
        let imported = Note::new("Imported only".to_string(), "New from backup".to_string());
        let backup_json =
            export_flat_collection_backup(&[replacement.clone(), imported.clone()]).unwrap();

        let pending =
            prepare_backup_import(std::slice::from_ref(&existing), backup_json.clone()).unwrap();

        assert_eq!(pending.backup_json, backup_json);
        assert_eq!(pending.preview.total_imported_notes, 2);
        assert_eq!(pending.preview.notes_to_add, 1);
        assert_eq!(pending.preview.notes_to_replace, 1);
        assert_eq!(pending.preview.selected_id, Some(replacement.id));
    }
}
