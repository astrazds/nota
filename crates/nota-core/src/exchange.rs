use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::Note;
use crate::backup::{
    BackupError, BackupHealth, BackupHealthRecord, BackupImportPreview, assess_backup_health,
    export_flat_collection_backup, flat_collection_notes, is_flat_collection_kind,
    prepare_backup_import,
};
use crate::transition::{
    ThemePreference, export_desktop_transition, import_desktop_transition,
    is_desktop_transition_kind, parse_desktop_transition,
};

/// What the session may do with a file the user already picked.
/// Constructed only by [`stage_import`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedImport {
    body: StagedBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StagedBody {
    MergeNotes {
        flat_json: String,
        preview: BackupImportPreview,
        deleted_notes_to_restore: usize,
        source_had_notebook_extras: bool,
    },
    ExactRestore {
        transition_json: String,
    },
}

pub struct ImportDestination<'a> {
    pub notes: &'a [Note],
    pub recently_deleted: &'a [Note],
    pub storage_recovery: bool,
}

pub struct NotebookExport {
    pub json: String,
    pub health: BackupHealthRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmCopy {
    pub title: String,
    pub body: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitPlan {
    MergeNotes { flat_json: String },
    ReplaceNotebook { transition_json: String },
}

#[derive(Debug)]
pub enum ExchangeError {
    Backup(BackupError),
    Transition(crate::transition::TransitionError),
    RecoveryRefusesCompleteRestore,
    Unreadable(serde_json::Error),
    UnsupportedKind(String),
}

impl fmt::Display for ExchangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backup(error) => write!(formatter, "{error}"),
            Self::Transition(error) => write!(formatter, "{error}"),
            Self::RecoveryRefusesCompleteRestore => write!(
                formatter,
                "Storage Recovery cannot replace this notebook. Import the file again to combine its active notes."
            ),
            Self::Unreadable(_) => {
                write!(formatter, "This file could not be read as a Nota backup.")
            }
            Self::UnsupportedKind(_) => write!(formatter, "This file is not a Nota backup."),
        }
    }
}

impl Error for ExchangeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Backup(error) => Some(error),
            Self::Transition(error) => Some(error),
            Self::Unreadable(error) => Some(error),
            Self::RecoveryRefusesCompleteRestore | Self::UnsupportedKind(_) => None,
        }
    }
}

#[derive(Deserialize)]
struct DocumentHeader {
    kind: String,
}

/// Writes a complete notebook and a new health clock.
/// The previous local clock is intentionally not a parameter.
pub fn export_notebook(
    notes: &[Note],
    recently_deleted: &[Note],
    theme: ThemePreference,
    now: DateTime<Utc>,
) -> Result<NotebookExport, ExchangeError> {
    let health = BackupHealthRecord {
        last_successful_export_at: now,
    };
    let json = export_desktop_transition(notes, recently_deleted, theme, Some(health))
        .map_err(ExchangeError::Transition)?;
    Ok(NotebookExport { json, health })
}

/// Sniff `kind`, then freeze one arm.
/// Flat always merges. A complete notebook exact-restores only when both
/// destination lists are empty and Storage Recovery is off. Otherwise only
/// its active notes are merged.
pub fn stage_import(
    json: &str,
    destination: ImportDestination<'_>,
) -> Result<StagedImport, ExchangeError> {
    let header: DocumentHeader = serde_json::from_str(json).map_err(ExchangeError::Unreadable)?;
    if is_flat_collection_kind(&header.kind) {
        return stage_merge(json.to_owned(), &destination, false);
    }
    if is_desktop_transition_kind(&header.kind) {
        let bundle = parse_desktop_transition(json).map_err(ExchangeError::Transition)?;
        let lists_empty = destination.notes.is_empty() && destination.recently_deleted.is_empty();
        if lists_empty && !destination.storage_recovery {
            return Ok(StagedImport {
                body: StagedBody::ExactRestore {
                    transition_json: json.to_owned(),
                },
            });
        }
        let flat_json =
            export_flat_collection_backup(&bundle.notes).map_err(ExchangeError::Backup)?;
        return stage_merge(flat_json, &destination, true);
    }
    Err(ExchangeError::UnsupportedKind(header.kind))
}

pub fn confirm_copy(staged: &StagedImport, now: DateTime<Utc>) -> ConfirmCopy {
    match &staged.body {
        StagedBody::MergeNotes {
            preview,
            deleted_notes_to_restore,
            source_had_notebook_extras,
            ..
        } => merge_copy(
            *preview,
            *deleted_notes_to_restore,
            *source_had_notebook_extras,
        ),
        StagedBody::ExactRestore { transition_json } => exact_restore_copy(transition_json, now),
    }
}

/// Re-check the frozen arm. This does not switch a merge into an exact restore
/// or the other way around.
pub fn prepare_commit(
    staged: &StagedImport,
    destination: ImportDestination<'_>,
) -> Result<CommitPlan, ExchangeError> {
    match &staged.body {
        StagedBody::MergeNotes { flat_json, .. } => {
            flat_collection_notes(flat_json).map_err(ExchangeError::Backup)?;
            Ok(CommitPlan::MergeNotes {
                flat_json: flat_json.clone(),
            })
        }
        StagedBody::ExactRestore { transition_json } => {
            if destination.storage_recovery {
                return Err(ExchangeError::RecoveryRefusesCompleteRestore);
            }
            import_desktop_transition(
                destination.notes,
                destination.recently_deleted,
                transition_json,
            )
            .map_err(ExchangeError::Transition)?;
            Ok(CommitPlan::ReplaceNotebook {
                transition_json: transition_json.clone(),
            })
        }
    }
}

fn stage_merge(
    flat_json: String,
    destination: &ImportDestination<'_>,
    source_had_notebook_extras: bool,
) -> Result<StagedImport, ExchangeError> {
    let notes = flat_collection_notes(&flat_json).map_err(ExchangeError::Backup)?;
    let pending = prepare_backup_import(
        destination.notes.iter().chain(destination.recently_deleted),
        flat_json.clone(),
    )
    .map_err(ExchangeError::Backup)?;
    let deleted_ids: HashSet<_> = destination
        .recently_deleted
        .iter()
        .map(|note| note.id)
        .collect();
    let deleted_notes_to_restore = notes
        .iter()
        .filter(|note| deleted_ids.contains(&note.id))
        .count();
    Ok(StagedImport {
        body: StagedBody::MergeNotes {
            flat_json,
            preview: pending.preview,
            deleted_notes_to_restore,
            source_had_notebook_extras,
        },
    })
}

fn merge_copy(
    preview: BackupImportPreview,
    deleted_notes_to_restore: usize,
    source_had_notebook_extras: bool,
) -> ConfirmCopy {
    let mut body = format!(
        "Notes in this file: {}\n\nNotes to add: {}\nNotes to replace: {}",
        preview.total_imported_notes, preview.notes_to_add, preview.notes_to_replace
    );
    if deleted_notes_to_restore > 0 {
        body.push_str(&format!(
            "\nRecently Deleted notes that will return: {deleted_notes_to_restore}"
        ));
    }
    if source_had_notebook_extras {
        body.push_str(
            "\n\nRecently Deleted, theme, and Backup Health from this file are not applied.",
        );
    }
    ConfirmCopy {
        title: "Import these notes?".to_string(),
        body,
        action: "Import".to_string(),
    }
}

fn exact_restore_copy(transition_json: &str, now: DateTime<Utc>) -> ConfirmCopy {
    let Ok(bundle) = parse_desktop_transition(transition_json) else {
        return ConfirmCopy {
            title: "Restore this backup?".to_string(),
            body: "This file could not be read as a Nota backup.".to_string(),
            action: "Restore".to_string(),
        };
    };
    let health = match bundle.backup_health {
        Some(record) => {
            let instant = format_instant(record.last_successful_export_at);
            match assess_backup_health(Some(record), now) {
                BackupHealth::Stale { .. } => format!(
                    "Backup Health in this file is already stale ({instant}). Restore replaces local Backup Health with that time."
                ),
                BackupHealth::Recent { .. } | BackupHealth::Missing => {
                    format!("Restore replaces local Backup Health with {instant}.")
                }
            }
        }
        None => "Restore clears local Backup Health.".to_string(),
    };
    ConfirmCopy {
        title: "Restore this backup?".to_string(),
        body: format!(
            "Active notes: {}\nRecently Deleted: {}\nTheme: {}\n\n{health}",
            bundle.notes.len(),
            bundle.recently_deleted_notes.len(),
            theme_label(bundle.theme)
        ),
        action: "Restore".to_string(),
    }
}

fn theme_label(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "Light",
        ThemePreference::Dark => "Dark",
        ThemePreference::System => "System",
    }
}

fn format_instant(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%d %H:%M UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::export_flat_collection_backup;
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn destination<'a>(
        notes: &'a [Note],
        recently_deleted: &'a [Note],
        storage_recovery: bool,
    ) -> ImportDestination<'a> {
        ImportDestination {
            notes,
            recently_deleted,
            storage_recovery,
        }
    }

    #[test]
    fn export_mints_a_complete_notebook_and_embeds_that_clock() {
        let active = Note::new("Active".to_string(), "Body".to_string());
        let deleted = Note::new("Deleted".to_string(), "Trash".to_string());
        let now = at(2026, 10, 14);
        let exported = export_notebook(
            std::slice::from_ref(&active),
            std::slice::from_ref(&deleted),
            ThemePreference::Dark,
            now,
        )
        .unwrap();

        assert_eq!(exported.health.last_successful_export_at, now);
        let value: serde_json::Value = serde_json::from_str(&exported.json).unwrap();
        assert_eq!(value["kind"], "nota.desktop_transition");
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["notes"][0]["title"], "Active");
        assert_eq!(value["recently_deleted_notes"][0]["title"], "Deleted");
        assert_eq!(
            value["backup_health"]["last_successful_export_at"],
            "2026-10-14T12:00:00Z"
        );
    }

    #[test]
    fn flat_file_always_merges_even_when_the_notebook_is_empty() {
        let imported = Note::new("From file".to_string(), "New".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
        let staged = stage_import(&json, destination(&[], &[], false)).unwrap();
        let copy = confirm_copy(&staged, at(2026, 10, 14));

        assert_eq!(copy.title, "Import these notes?");
        assert_eq!(copy.action, "Import");
        assert_eq!(
            copy.body,
            "Notes in this file: 1\n\nNotes to add: 1\nNotes to replace: 0"
        );
        assert_eq!(
            prepare_commit(&staged, destination(&[], &[], false)).unwrap(),
            CommitPlan::MergeNotes { flat_json: json }
        );
    }

    #[test]
    fn legacy_flat_kind_merges_and_counts_a_deleted_note_as_a_return() {
        let note = Note::new("Returned".to_string(), "Body".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&note))
            .unwrap()
            .replace("nota.flat_collection", "noter.flat_collection");
        let staged =
            stage_import(&json, destination(&[], std::slice::from_ref(&note), false)).unwrap();
        let copy = confirm_copy(&staged, at(2026, 10, 14));

        assert_eq!(
            copy.body,
            "Notes in this file: 1\n\nNotes to add: 0\nNotes to replace: 1\nRecently Deleted notes that will return: 1"
        );
    }

    #[test]
    fn complete_file_into_an_empty_notebook_stages_exact_restore() {
        let active = Note::new("Active".to_string(), "Body".to_string());
        let deleted = Note::new("Deleted".to_string(), "Trash".to_string());
        let health = BackupHealthRecord {
            last_successful_export_at: at(2026, 10, 13),
        };
        let json = export_desktop_transition(
            std::slice::from_ref(&active),
            std::slice::from_ref(&deleted),
            ThemePreference::Dark,
            Some(health),
        )
        .unwrap();
        let now = at(2026, 10, 14);
        let staged = stage_import(&json, destination(&[], &[], false)).unwrap();
        let copy = confirm_copy(&staged, now);

        assert_eq!(copy.title, "Restore this backup?");
        assert_eq!(copy.action, "Restore");
        assert_eq!(
            copy.body,
            "Active notes: 1\nRecently Deleted: 1\nTheme: Dark\n\nRestore replaces local Backup Health with 2026-10-13 12:00 UTC."
        );
        assert_eq!(
            prepare_commit(&staged, destination(&[], &[], false)).unwrap(),
            CommitPlan::ReplaceNotebook {
                transition_json: json
            }
        );
    }

    #[test]
    fn complete_file_with_stale_or_missing_health_says_which_transfer_happens() {
        let now = at(2026, 10, 14);
        let stale = BackupHealthRecord {
            last_successful_export_at: at(2026, 9, 1),
        };
        let stale_json =
            export_desktop_transition(&[], &[], ThemePreference::Light, Some(stale)).unwrap();
        let stale_copy = confirm_copy(
            &stage_import(&stale_json, destination(&[], &[], false)).unwrap(),
            now,
        );
        assert_eq!(
            stale_copy.body,
            "Active notes: 0\nRecently Deleted: 0\nTheme: Light\n\nBackup Health in this file is already stale (2026-09-01 12:00 UTC). Restore replaces local Backup Health with that time."
        );

        let clear_json =
            export_desktop_transition(&[], &[], ThemePreference::System, None).unwrap();
        let clear_copy = confirm_copy(
            &stage_import(&clear_json, destination(&[], &[], false)).unwrap(),
            now,
        );
        assert_eq!(
            clear_copy.body,
            "Active notes: 0\nRecently Deleted: 0\nTheme: System\n\nRestore clears local Backup Health."
        );
    }

    #[test]
    fn complete_file_into_a_nonempty_notebook_merges_active_notes_only() {
        let active = Note::new("Active".to_string(), "Body".to_string());
        let deleted = Note::new("File trash".to_string(), "Drop me".to_string());
        let local = Note::new("Local".to_string(), "Stay".to_string());
        let json = export_desktop_transition(
            std::slice::from_ref(&active),
            std::slice::from_ref(&deleted),
            ThemePreference::Dark,
            Some(BackupHealthRecord {
                last_successful_export_at: at(2026, 10, 1),
            }),
        )
        .unwrap();
        let staged =
            stage_import(&json, destination(std::slice::from_ref(&local), &[], false)).unwrap();
        let copy = confirm_copy(&staged, at(2026, 10, 14));
        let CommitPlan::MergeNotes { flat_json } = prepare_commit(
            &staged,
            destination(std::slice::from_ref(&local), &[], false),
        )
        .unwrap() else {
            panic!("a nonempty notebook keeps the merge arm");
        };

        assert_eq!(copy.action, "Import");
        assert!(copy.body.contains("Notes to add: 1"));
        assert!(copy.body.contains(
            "Recently Deleted, theme, and Backup Health from this file are not applied."
        ));
        assert!(flat_json.contains("Active"));
        assert!(!flat_json.contains("File trash"));
        assert!(!flat_json.contains("backup_health"));
    }

    #[test]
    fn storage_recovery_merges_a_complete_file_even_when_both_lists_are_empty() {
        let active = Note::new("Active".to_string(), "Body".to_string());
        let deleted = Note::new("File trash".to_string(), "Drop me".to_string());
        let json = export_desktop_transition(
            std::slice::from_ref(&active),
            std::slice::from_ref(&deleted),
            ThemePreference::Dark,
            None,
        )
        .unwrap()
        .replace("nota.desktop_transition", "noter.desktop_transition");
        let staged = stage_import(&json, destination(&[], &[], true)).unwrap();
        let CommitPlan::MergeNotes { flat_json } =
            prepare_commit(&staged, destination(&[], &[], true)).unwrap()
        else {
            panic!("Storage Recovery never stages an exact restore");
        };

        assert_eq!(confirm_copy(&staged, at(2026, 10, 14)).action, "Import");
        assert!(flat_json.contains("Active"));
        assert!(!flat_json.contains("File trash"));
        assert!(flat_json.contains("nota.flat_collection"));
    }

    #[test]
    fn prepare_commit_does_not_switch_arms_when_the_notebook_changes() {
        let active = Note::new("Active".to_string(), "Body".to_string());
        let json = export_desktop_transition(
            std::slice::from_ref(&active),
            &[],
            ThemePreference::Light,
            None,
        )
        .unwrap();
        let staged = stage_import(&json, destination(&[], &[], false)).unwrap();
        let error = prepare_commit(
            &staged,
            destination(std::slice::from_ref(&active), &[], false),
        )
        .expect_err("exact restore stays exact and fails when a note appears");
        assert_eq!(
            error.to_string(),
            "The notebook is no longer empty, so this restore was not applied. Import the file again to combine notes."
        );

        let recovery_error = prepare_commit(&staged, destination(&[], &[], true))
            .expect_err("an already staged exact restore cannot commit during Storage Recovery");
        assert_eq!(
            recovery_error.to_string(),
            "Storage Recovery cannot replace this notebook. Import the file again to combine its active notes."
        );

        let flat = export_flat_collection_backup(std::slice::from_ref(&active)).unwrap();
        let merge = stage_import(
            &flat,
            destination(std::slice::from_ref(&active), &[], false),
        )
        .unwrap();
        assert!(matches!(
            prepare_commit(&merge, destination(&[], &[], false)).unwrap(),
            CommitPlan::MergeNotes { .. }
        ));
    }

    #[test]
    fn legacy_fixtures_stage_through_the_same_arms() {
        let flat = include_str!("../tests/fixtures/flat-collection-backup-v1.json");
        let transition = include_str!("../tests/fixtures/desktop-transition-v1.json");
        let local = Note::new("Local".to_string(), "Stay".to_string());

        let flat_plan = prepare_commit(
            &stage_import(flat, destination(std::slice::from_ref(&local), &[], false)).unwrap(),
            destination(std::slice::from_ref(&local), &[], false),
        )
        .unwrap();
        let CommitPlan::MergeNotes { flat_json } = flat_plan else {
            panic!("a legacy flat file merges");
        };
        assert!(flat_json.contains("Existing Backup"));

        let exact = prepare_commit(
            &stage_import(transition, destination(&[], &[], false)).unwrap(),
            destination(&[], &[], false),
        )
        .unwrap();
        assert!(matches!(exact, CommitPlan::ReplaceNotebook { .. }));

        let recovery = prepare_commit(
            &stage_import(transition, destination(&[], &[], true)).unwrap(),
            destination(&[], &[], true),
        )
        .unwrap();
        let CommitPlan::MergeNotes { flat_json } = recovery else {
            panic!("Storage Recovery merges a legacy complete file");
        };
        assert!(flat_json.contains("Desktop handoff"));
        assert!(!flat_json.contains("Recover later"));
    }

    #[test]
    fn an_unknown_kind_is_refused_without_echoing_the_wire_value() {
        let json = r#"{"version":1,"kind":"nota.notebook_backup","notes":[]}"#;
        let error = stage_import(json, destination(&[], &[], false)).unwrap_err();
        assert_eq!(error.to_string(), "This file is not a Nota backup.");
        assert!(!error.to_string().contains("nota.notebook_backup"));

        let unreadable = stage_import("{", destination(&[], &[], false)).unwrap_err();
        assert_eq!(
            unreadable.to_string(),
            "This file could not be read as a Nota backup."
        );
    }
}
