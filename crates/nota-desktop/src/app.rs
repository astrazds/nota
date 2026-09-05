use chrono::{DateTime, Utc};
use nota_core::NoteWorkspace;
use nota_core::backup::{
    BackupHealth, BackupHealthRecord, PendingBackupImport, assess_backup_health,
    import_flat_collection_backup, prepare_backup_import,
};
use nota_core::editor_view::EditorViewMode;
use nota_core::markdown_editing::{ByteSelection, MarkdownCommand, apply_markdown_command};
use nota_core::note_list_interaction::{NoteListInteraction, NoteListRenderModel};
use nota_core::responsive_navigation::{ViewportClass, normalize_view_mode};
use nota_core::tag_rules::{TagSuggestion, parse_tags_input, suggest_existing_tags};
use nota_core::transition::{ThemePreference, TransitionError, import_desktop_transition};
use uuid::Uuid;

use crate::storage::CollectionEnvelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Saved,
    Saving,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationTone {
    Progress,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalNotification {
    pub message: String,
    pub tone: NotificationTone,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    QuickCapture,
    SelectNote(Uuid),
    UpdateTitle(String),
    UpdateContent(String),
    UpdateTags(String),
    ApplyFormatting {
        selection: ByteSelection,
        command: MarkdownCommand,
    },
    EditSearch(String),
    CommitSearch,
    SelectTag(String),
    ClearTag,
    TogglePin(Uuid),
    RequestDelete(Uuid),
    CancelDelete,
    ConfirmDelete,
    RestoreRecentlyDeleted(Uuid),
    PermanentlyDelete(Uuid),
    RequestClearAll,
    CancelClearAll,
    ConfirmClearAll,
    SetViewMode(EditorViewMode),
    Resize(f64),
    ToggleNavigation,
    RestorePreviousSnapshot,
    StartEmptyAfterRecovery,
    FlushPersistence,
    PersistenceComplete(u64),
    PersistenceFailed(String),
    RequestBackupExport,
    RequestBackupImport,
    RequestTransitionExport,
    RequestTransitionImport,
    RequestDiagnostics,
    ImportBackupJson(String),
    ConfirmBackupImport,
    CancelBackupImport,
    ImportTransitionJson(String),
    BackupExported(DateTime<Utc>),
    ToggleTheme,
    OperationSucceeded(String),
    OperationFailed(String),
    DismissNotification(u64),
    StartEditTags,
    FinishEditTags,
    AcceptTagSuggestion(String),
}

#[derive(Debug)]
pub struct AppModel {
    pub workspace: NoteWorkspace,
    pub note_list: NoteListInteraction,
    pub theme: ThemePreference,
    pub view_mode: EditorViewMode,
    pub viewport: ViewportClass,
    pub note_list_visible: bool,
    pub save_status: SaveStatus,
    pub notification: Option<GlobalNotification>,
    pub backup_health: Option<BackupHealthRecord>,
    pending_backup_import: Option<PendingBackupImport>,
    storage_recovery: bool,
    notification_generation: u64,
    editing_tags: bool,
    revision: u64,
}

impl AppModel {
    pub fn new(
        collection: CollectionEnvelope,
        theme: ThemePreference,
        backup_health: Option<BackupHealthRecord>,
    ) -> Self {
        Self {
            workspace: NoteWorkspace::new_with_recently_deleted(
                collection.notes,
                collection.recently_deleted_notes,
            ),
            note_list: NoteListInteraction::default(),
            theme,
            view_mode: EditorViewMode::Write,
            viewport: ViewportClass::Wide,
            note_list_visible: true,
            save_status: SaveStatus::Saved,
            notification: None,
            backup_health,
            pending_backup_import: None,
            storage_recovery: false,
            notification_generation: 0,
            editing_tags: false,
            revision: 0,
        }
    }

    pub fn apply(&mut self, message: AppMsg) -> bool {
        if self.storage_recovery && !Self::message_allowed_during_storage_recovery(&message) {
            return false;
        }
        let changed = match message {
            AppMsg::QuickCapture => {
                self.workspace.create_note();
                true
            }
            AppMsg::SelectNote(id) => {
                let selected = self.workspace.select_note(id);
                if selected && self.viewport == ViewportClass::Compact {
                    self.note_list_visible = false;
                }
                false
            }
            AppMsg::UpdateTitle(title) => self.workspace.update_selected_title(title),
            AppMsg::UpdateContent(content) => self.workspace.update_selected_content(content),
            AppMsg::UpdateTags(tags) => {
                self.workspace.update_selected_tags(parse_tags_input(&tags))
            }
            AppMsg::ApplyFormatting { selection, command } => {
                let Some(note) = self.workspace.selected_note() else {
                    return false;
                };
                let formatted = apply_markdown_command(&note.content, selection, command);
                self.workspace.update_selected_content(formatted.content)
            }
            AppMsg::EditSearch(search) => {
                self.note_list.edit_search(search);
                false
            }
            AppMsg::CommitSearch => {
                self.note_list.commit_search();
                false
            }
            AppMsg::SelectTag(tag) => {
                self.note_list.select_tag(tag);
                false
            }
            AppMsg::ClearTag => {
                self.note_list.clear_tag();
                false
            }
            AppMsg::TogglePin(id) => self.workspace.toggle_pin(id),
            AppMsg::RequestDelete(id) => self.workspace.request_delete(id) && false,
            AppMsg::CancelDelete => {
                self.workspace.cancel_delete();
                false
            }
            AppMsg::ConfirmDelete => self.workspace.confirm_delete(),
            AppMsg::RestoreRecentlyDeleted(id) => self.workspace.restore_recently_deleted(id),
            AppMsg::PermanentlyDelete(id) => self.workspace.permanently_clear_recently_deleted(id),
            AppMsg::RequestClearAll => self.workspace.request_clear_all_recently_deleted() && false,
            AppMsg::CancelClearAll => {
                self.workspace.cancel_clear_all_recently_deleted();
                false
            }
            AppMsg::ConfirmClearAll => self.workspace.confirm_clear_all_recently_deleted(),
            AppMsg::SetViewMode(mode) => {
                self.view_mode = normalize_view_mode(self.viewport, mode);
                false
            }
            AppMsg::Resize(width) => {
                // GTK may notify width=0 before the window is mapped. Treating that
                // as Compact hides the sidebar/editor exclusively and sticks there.
                if width <= 0.0 {
                    return false;
                }
                self.viewport = ViewportClass::from_width(width);
                if self.viewport == ViewportClass::Wide {
                    self.note_list_visible = true;
                }
                self.view_mode = normalize_view_mode(self.viewport, self.view_mode);
                false
            }
            AppMsg::ToggleNavigation => {
                if self.viewport == ViewportClass::Compact {
                    self.note_list_visible = !self.note_list_visible;
                }
                false
            }
            AppMsg::RestorePreviousSnapshot | AppMsg::StartEmptyAfterRecovery => false,
            AppMsg::FlushPersistence => false,
            AppMsg::PersistenceComplete(revision) => {
                if revision == self.revision {
                    self.save_status = SaveStatus::Saved;
                }
                false
            }
            AppMsg::PersistenceFailed(error) => {
                self.save_status = SaveStatus::Failed;
                self.set_notification(error, NotificationTone::Error);
                false
            }
            AppMsg::BackupExported(exported_at) => {
                self.backup_health = Some(BackupHealthRecord {
                    last_successful_export_at: exported_at,
                });
                self.set_notification("Backup exported", NotificationTone::Success);
                false
            }
            AppMsg::ToggleTheme => {
                self.theme = match self.theme {
                    ThemePreference::Dark => ThemePreference::Light,
                    ThemePreference::Light | ThemePreference::System => ThemePreference::Dark,
                };
                false
            }
            AppMsg::OperationSucceeded(message) => {
                self.set_notification(message, NotificationTone::Success);
                false
            }
            AppMsg::OperationFailed(message) => {
                self.set_notification(message, NotificationTone::Error);
                false
            }
            AppMsg::DismissNotification(generation) => {
                if generation == self.notification_generation {
                    self.notification = None;
                }
                false
            }
            AppMsg::StartEditTags => {
                self.editing_tags = true;
                false
            }
            AppMsg::FinishEditTags => {
                self.editing_tags = false;
                false
            }
            AppMsg::AcceptTagSuggestion(input) => {
                let Some(suggestion) = self.tag_suggestions(&input).into_iter().next() else {
                    return false;
                };
                self.workspace
                    .update_selected_tags(parse_tags_input(&suggestion.completed_input))
            }
            AppMsg::ImportBackupJson(json) => {
                match prepare_backup_import(self.workspace.notes(), json) {
                    Ok(pending) => {
                        self.pending_backup_import = Some(pending);
                        self.set_notification("Backup ready", NotificationTone::Success);
                    }
                    Err(error) => {
                        self.pending_backup_import = None;
                        self.set_notification(error.to_string(), NotificationTone::Error);
                    }
                }
                false
            }
            AppMsg::ConfirmBackupImport => {
                let changed = self.confirm_pending_backup_import();
                if changed {
                    self.storage_recovery = false;
                }
                changed
            }
            AppMsg::CancelBackupImport => {
                self.pending_backup_import = None;
                self.set_notification("Backup import cancelled", NotificationTone::Progress);
                false
            }
            AppMsg::RequestBackupExport
            | AppMsg::RequestBackupImport
            | AppMsg::RequestTransitionExport
            | AppMsg::RequestTransitionImport
            | AppMsg::RequestDiagnostics
            | AppMsg::ImportTransitionJson(_) => false,
        };

        if changed {
            self.revision = self.revision.saturating_add(1);
            self.save_status = SaveStatus::Saving;
        }
        changed
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn pending_backup_import(&self) -> Option<&PendingBackupImport> {
        self.pending_backup_import.as_ref()
    }

    pub fn set_storage_recovery(&mut self, active: bool) {
        self.storage_recovery = active;
    }

    pub fn is_in_storage_recovery(&self) -> bool {
        self.storage_recovery
    }

    pub fn notification_generation(&self) -> u64 {
        self.notification_generation
    }

    pub fn backup_health_status(&self, now: DateTime<Utc>) -> BackupHealth {
        assess_backup_health(self.backup_health, now)
    }

    pub fn is_editing_tags(&self) -> bool {
        self.editing_tags
    }

    pub fn tag_suggestions(&self, input: &str) -> Vec<TagSuggestion> {
        let selected = self.workspace.selected_note();
        suggest_existing_tags(self.workspace.notes(), selected.as_ref(), input)
    }

    pub fn backup_health_label(&self, now: DateTime<Utc>) -> &'static str {
        match self.backup_health_status(now) {
            BackupHealth::Missing => "No backup yet",
            BackupHealth::Recent { .. } => "Up to date",
            BackupHealth::Stale { .. } => "Backup stale",
        }
    }

    fn set_notification(&mut self, message: impl Into<String>, tone: NotificationTone) {
        self.notification_generation = self.notification_generation.saturating_add(1);
        self.notification = Some(GlobalNotification {
            message: message.into(),
            tone,
        });
    }

    pub fn collection(&self) -> CollectionEnvelope {
        CollectionEnvelope::new(
            self.workspace.notes().to_vec(),
            self.workspace.recently_deleted_notes().to_vec(),
        )
    }

    pub fn note_list_render_model(&self) -> NoteListRenderModel {
        self.note_list
            .render_model(self.workspace.notes(), self.workspace.selected_id())
    }

    pub fn replace_loaded_collection(&mut self, collection: CollectionEnvelope) {
        self.workspace = NoteWorkspace::new_with_recently_deleted(
            collection.notes,
            collection.recently_deleted_notes,
        );
        self.save_status = SaveStatus::Saved;
        self.notification = None;
    }

    fn confirm_pending_backup_import(&mut self) -> bool {
        let Some(pending) = self.pending_backup_import.take() else {
            return false;
        };
        match self.import_backup(&pending.backup_json) {
            Ok(()) => true,
            Err(error) => {
                self.pending_backup_import = Some(pending);
                self.set_notification(error.to_string(), NotificationTone::Error);
                false
            }
        }
    }

    pub fn import_backup(&mut self, json: &str) -> Result<(), nota_core::backup::BackupError> {
        let mut notes = self.workspace.notes().to_vec();
        let deleted = self.workspace.recently_deleted_notes().to_vec();
        let imported = import_flat_collection_backup(&mut notes, json)?;
        self.workspace = NoteWorkspace::new_with_recently_deleted(notes, deleted);
        if let Some(id) = imported.selected_id {
            self.workspace.select_note(id);
        }
        self.set_notification("Backup imported", NotificationTone::Success);
        Ok(())
    }

    pub fn import_transition(&mut self, json: &str) -> Result<(), TransitionError> {
        let restored = import_desktop_transition(
            self.workspace.notes(),
            self.workspace.recently_deleted_notes(),
            json,
        )?;
        self.workspace = NoteWorkspace::new_with_recently_deleted(
            restored.notes,
            restored.recently_deleted_notes,
        );
        self.theme = restored.theme;
        self.backup_health = restored.backup_health;
        self.mark_external_change("Desktop transition restored");
        Ok(())
    }

    fn message_allowed_during_storage_recovery(message: &AppMsg) -> bool {
        matches!(
            message,
            AppMsg::RestorePreviousSnapshot
                | AppMsg::StartEmptyAfterRecovery
                | AppMsg::RequestBackupImport
                | AppMsg::ImportBackupJson(_)
                | AppMsg::ConfirmBackupImport
                | AppMsg::CancelBackupImport
                | AppMsg::RequestDiagnostics
                | AppMsg::FlushPersistence
                | AppMsg::PersistenceComplete(_)
                | AppMsg::PersistenceFailed(_)
                | AppMsg::OperationSucceeded(_)
                | AppMsg::OperationFailed(_)
                | AppMsg::DismissNotification(_)
                | AppMsg::Resize(_)
        )
    }

    fn mark_external_change(&mut self, message: &str) {
        self.revision = self.revision.saturating_add(1);
        self.save_status = SaveStatus::Saving;
        self.set_notification(message, NotificationTone::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use nota_core::transition::export_desktop_transition;

    #[test]
    fn quick_capture_edit_search_delete_restore_and_clear_follow_messages() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert!(app.apply(AppMsg::QuickCapture));
        let id = app.workspace.selected_id().unwrap();
        assert!(app.apply(AppMsg::UpdateTitle("Roadmap".to_string())));
        assert!(app.apply(AppMsg::UpdateContent("Ship native app".to_string())));
        assert!(app.apply(AppMsg::UpdateTags("Work, Rust".to_string())));

        app.apply(AppMsg::EditSearch("native".to_string()));
        app.apply(AppMsg::CommitSearch);
        assert_eq!(app.note_list_render_model().projection.rows.len(), 1);

        app.apply(AppMsg::RequestDelete(id));
        assert!(app.apply(AppMsg::ConfirmDelete));
        assert!(app.workspace.notes().is_empty());
        assert_eq!(app.workspace.recently_deleted_notes().len(), 1);

        assert!(app.apply(AppMsg::RestoreRecentlyDeleted(id)));
        app.apply(AppMsg::RequestDelete(id));
        app.apply(AppMsg::ConfirmDelete);
        app.apply(AppMsg::RequestClearAll);
        assert!(app.apply(AppMsg::ConfirmClearAll));
        assert!(app.workspace.recently_deleted_notes().is_empty());
    }

    #[test]
    fn resize_ignores_non_positive_widths_so_unmapped_windows_stay_dual_pane() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        assert_eq!(app.viewport, ViewportClass::Wide);
        assert!(app.note_list_visible);

        app.apply(AppMsg::Resize(0.0));
        assert_eq!(app.viewport, ViewportClass::Wide);
        assert!(app.note_list_visible);

        app.apply(AppMsg::Resize(-1.0));
        assert_eq!(app.viewport, ViewportClass::Wide);
    }

    #[test]
    fn split_view_normalizes_when_the_window_becomes_narrow() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        app.apply(AppMsg::SetViewMode(EditorViewMode::Split));
        assert_eq!(app.view_mode, EditorViewMode::Split);

        app.apply(AppMsg::Resize(600.0));

        assert_eq!(app.viewport, ViewportClass::Compact);
        assert_eq!(app.view_mode, EditorViewMode::Write);
    }

    #[test]
    fn compact_navigation_uses_full_width_surfaces_and_selection_returns_to_writing() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        app.apply(AppMsg::QuickCapture);
        let id = app.workspace.selected_id().unwrap();
        app.apply(AppMsg::Resize(700.0));

        app.apply(AppMsg::ToggleNavigation);
        assert!(!app.note_list_visible);
        app.apply(AppMsg::ToggleNavigation);
        assert!(app.note_list_visible);

        app.apply(AppMsg::SelectNote(id));
        assert!(!app.note_list_visible);
        app.apply(AppMsg::Resize(1200.0));
        assert!(app.note_list_visible);
    }

    #[test]
    fn edit_tags_suggests_other_collection_tags_for_the_current_fragment() {
        let mut selected = nota_core::Note::new("Selected".to_string(), String::new());
        selected.tags = vec!["Work".to_string()];
        let mut other = nota_core::Note::new("Other".to_string(), String::new());
        other.tags = vec!["Research".to_string(), "Work".to_string()];
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![selected.clone(), other], Vec::new()),
            ThemePreference::Light,
            None,
        );
        app.apply(AppMsg::SelectNote(selected.id));

        let suggestions = app.tag_suggestions("Work, r");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].label, "Research");
        assert_eq!(suggestions[0].completed_input, "Work, Research");
    }

    #[test]
    fn accepting_a_tag_suggestion_applies_the_first_match_to_the_selected_note() {
        let mut selected = nota_core::Note::new("Selected".to_string(), String::new());
        selected.tags = vec!["Work".to_string()];
        let mut other = nota_core::Note::new("Other".to_string(), String::new());
        other.tags = vec!["Research".to_string()];
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![selected.clone(), other], Vec::new()),
            ThemePreference::Light,
            None,
        );
        app.apply(AppMsg::SelectNote(selected.id));

        assert!(app.apply(AppMsg::AcceptTagSuggestion("Work, r".to_string())));
        assert_eq!(
            app.workspace.selected_note().unwrap().tags,
            vec!["Work".to_string(), "Research".to_string()]
        );
    }

    #[test]
    fn selecting_a_note_keeps_note_list_row_order() {
        let mut older = nota_core::Note::new("Older".to_string(), String::new());
        let mut newer = nota_core::Note::new("Newer".to_string(), String::new());
        older.last_modified = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        newer.last_modified = chrono::Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap();
        let older_id = older.id;
        let newer_id = newer.id;
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![older, newer], Vec::new()),
            ThemePreference::Light,
            None,
        );
        app.apply(AppMsg::SelectNote(older_id));
        let before: Vec<_> = app
            .note_list_render_model()
            .projection
            .rows
            .iter()
            .map(|row| row.id)
            .collect();

        app.apply(AppMsg::SelectNote(newer_id));

        let after: Vec<_> = app
            .note_list_render_model()
            .projection
            .rows
            .iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(before, after);
        assert_eq!(after, vec![newer_id, older_id]);
        assert_eq!(app.workspace.selected_id(), Some(newer_id));
    }

    #[test]
    fn theme_changes_do_not_create_a_collection_revision() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);

        app.apply(AppMsg::ToggleTheme);

        assert_eq!(app.theme, ThemePreference::Dark);
        assert_eq!(app.revision(), 0);
        assert_eq!(app.save_status, SaveStatus::Saved);
    }

    #[test]
    fn transition_restore_is_exact_on_first_run_and_rejected_after_native_data_exists() {
        let imported = nota_core::Note::new("From web".to_string(), "Exact state".to_string());
        let json = export_desktop_transition(
            std::slice::from_ref(&imported),
            &[],
            ThemePreference::Dark,
            None,
        )
        .unwrap();
        let mut first_run =
            AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);

        first_run.import_transition(&json).unwrap();

        assert_eq!(first_run.workspace.notes(), std::slice::from_ref(&imported));
        assert_eq!(first_run.theme, ThemePreference::Dark);
        assert_eq!(first_run.revision(), 1);

        let before = first_run.collection();
        let revision = first_run.revision();
        let error = first_run
            .import_transition(&json)
            .expect_err("native data must block an exact transition restore");
        assert!(matches!(error, TransitionError::CollectionNotEmpty));
        assert_eq!(first_run.collection(), before);
        assert_eq!(first_run.revision(), revision);
    }

    #[test]
    fn backup_json_prepares_import_preview_without_mutating_the_collection() {
        use nota_core::backup::export_flat_collection_backup;

        let existing = nota_core::Note::new("Local".to_string(), "Keep me".to_string());
        let imported = nota_core::Note::new("From backup".to_string(), "New".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
            ThemePreference::Light,
            None,
        );

        app.apply(AppMsg::ImportBackupJson(json));

        assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
        let pending = app
            .pending_backup_import()
            .expect("Backup Import Preview should be pending");
        assert_eq!(pending.preview.notes_to_add, 1);
        assert_eq!(pending.preview.notes_to_replace, 0);
        assert_eq!(pending.preview.total_imported_notes, 1);
        assert_eq!(app.revision(), 0);
    }

    #[test]
    fn backup_import_preview_counts_same_identity_notes_as_replacements() {
        use nota_core::backup::export_flat_collection_backup;

        let mut existing = nota_core::Note::new("Local".to_string(), "Keep me".to_string());
        let replacement = {
            let mut note = existing.clone();
            note.title = "Replaced".to_string();
            note.content = "Updated".to_string();
            note
        };
        existing.title = "Original".to_string();
        let json = export_flat_collection_backup(std::slice::from_ref(&replacement)).unwrap();
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
            ThemePreference::Light,
            None,
        );

        app.apply(AppMsg::ImportBackupJson(json));

        let pending = app
            .pending_backup_import()
            .expect("Backup Import Preview should be pending");
        assert_eq!(pending.preview.notes_to_add, 0);
        assert_eq!(pending.preview.notes_to_replace, 1);
        assert_eq!(app.workspace.notes()[0].title, "Original");
        assert!(app.apply(AppMsg::ConfirmBackupImport));
        assert_eq!(app.workspace.notes()[0].title, "Replaced");
    }

    #[test]
    fn storage_recovery_blocks_quick_capture_and_search_until_cleared() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.set_storage_recovery(true);

        assert!(!app.apply(AppMsg::QuickCapture));
        assert!(app.workspace.notes().is_empty());

        app.apply(AppMsg::EditSearch("secret".to_string()));
        app.apply(AppMsg::CommitSearch);
        assert!(app.note_list.search_input().is_empty());
        assert!(app.note_list.committed_search().is_empty());

        app.set_storage_recovery(false);
        assert!(app.apply(AppMsg::QuickCapture));
        assert_eq!(app.workspace.notes().len(), 1);
    }

    #[test]
    fn quick_capture_requests_note_title_focus() {
        use nota_core::note_workspace::FocusIntent;

        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert!(app.apply(AppMsg::QuickCapture));
        assert_eq!(app.workspace.focus_intent(), FocusIntent::NoteTitle);
    }

    #[test]
    fn tag_editing_is_opt_in_and_does_not_revise_the_collection() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.apply(AppMsg::StartEditTags);
        assert!(app.is_editing_tags());
        assert_eq!(app.revision(), 0);
        app.apply(AppMsg::FinishEditTags);
        assert!(!app.is_editing_tags());
    }

    #[test]
    fn storage_recovery_allows_backup_import_preview_and_merge() {
        use nota_core::backup::export_flat_collection_backup;

        let imported = nota_core::Note::new("Recovered".to_string(), "From Backup".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.set_storage_recovery(true);

        app.apply(AppMsg::ImportBackupJson(json));
        assert!(app.pending_backup_import().is_some());
        assert!(app.apply(AppMsg::ConfirmBackupImport));
        assert!(!app.is_in_storage_recovery());
        assert_eq!(app.workspace.notes(), std::slice::from_ref(&imported));
    }

    #[test]
    fn global_notifications_can_be_dismissed_without_mutating_notes() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.apply(AppMsg::OperationSucceeded("Backup exported".to_string()));
        assert!(app.notification.is_some());
        let generation = app.notification_generation();
        app.apply(AppMsg::DismissNotification(generation));
        assert!(app.notification.is_none());
        assert_eq!(app.revision(), 0);
    }

    #[test]
    fn backup_health_label_is_actionable_for_missing_recent_and_stale() {
        use chrono::TimeZone;

        let missing = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        let now = chrono::Utc.with_ymd_and_hms(2026, 9, 4, 12, 0, 0).unwrap();
        assert_eq!(missing.backup_health_label(now), "No backup yet");

        let recent = AppModel::new(
            CollectionEnvelope::empty(),
            ThemePreference::Light,
            Some(BackupHealthRecord {
                last_successful_export_at: now,
            }),
        );
        assert_eq!(recent.backup_health_label(now), "Up to date");

        let stale = AppModel::new(
            CollectionEnvelope::empty(),
            ThemePreference::Light,
            Some(BackupHealthRecord {
                last_successful_export_at: now - chrono::Duration::days(15),
            }),
        );
        assert_eq!(stale.backup_health_label(now), "Backup stale");
    }

    #[test]
    fn search_commit_distinguishes_filtered_empty_from_empty_collection() {
        use nota_core::note_list_interaction::NoteListDisplayState;

        let note = nota_core::Note::new("Roadmap".to_string(), "Ship native app".to_string());
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![note], Vec::new()),
            ThemePreference::Light,
            None,
        );
        app.apply(AppMsg::EditSearch("zzzz".to_string()));
        assert_eq!(
            app.note_list_render_model().display_state,
            NoteListDisplayState::Rows
        );
        app.apply(AppMsg::CommitSearch);
        let model = app.note_list_render_model();
        assert_eq!(model.display_state, NoteListDisplayState::FilteredEmpty);
        assert!(model.result_status.is_some());
        assert!(
            model
                .filtered_empty_message
                .title
                .contains("No notes match search")
        );

        let empty = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert_eq!(
            empty.note_list_render_model().display_state,
            NoteListDisplayState::EmptyCollection
        );
    }

    #[test]
    fn selecting_a_tag_filters_the_note_list_without_a_match_label() {
        let mut work = nota_core::Note::new("Sprint".to_string(), "body phrase unique".to_string());
        work.tags = vec!["Work".to_string()];
        let personal = nota_core::Note::new("Groceries".to_string(), "apples".to_string());
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![work, personal], Vec::new()),
            ThemePreference::Light,
            None,
        );

        app.apply(AppMsg::SelectTag("Work".to_string()));
        let filtered = app.note_list_render_model();
        assert_eq!(filtered.projection.rows.len(), 1);
        assert_eq!(app.note_list.active_tag(), Some("Work"));

        app.apply(AppMsg::ClearTag);
        assert!(app.note_list.active_tag().is_none());
        assert_eq!(app.note_list_render_model().projection.rows.len(), 2);

        app.apply(AppMsg::EditSearch("unique".to_string()));
        app.apply(AppMsg::CommitSearch);
        let snippet = &app.note_list_render_model().projection.rows[0];
        assert!(snippet.uses_match_snippet);
        assert!(!snippet.preview.starts_with("Match:"));
    }

    #[test]
    fn confirming_backup_import_preview_applies_merge_import() {
        use nota_core::backup::export_flat_collection_backup;

        let existing = nota_core::Note::new("Local".to_string(), "Keep me".to_string());
        let imported = nota_core::Note::new("From backup".to_string(), "New".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
            ThemePreference::Light,
            None,
        );

        app.apply(AppMsg::ImportBackupJson(json));
        assert!(app.apply(AppMsg::ConfirmBackupImport));

        assert_eq!(app.workspace.notes().len(), 2);
        assert!(app.pending_backup_import().is_none());
        assert_eq!(app.workspace.selected_id(), Some(imported.id));
        assert_eq!(app.save_status, SaveStatus::Saving);
    }

    #[test]
    fn canceling_backup_import_preview_leaves_the_collection_unchanged() {
        use nota_core::backup::export_flat_collection_backup;

        let existing = nota_core::Note::new("Local".to_string(), "Keep me".to_string());
        let imported = nota_core::Note::new("From backup".to_string(), "New".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
            ThemePreference::Light,
            None,
        );

        app.apply(AppMsg::ImportBackupJson(json));
        app.apply(AppMsg::CancelBackupImport);

        assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
        assert!(app.pending_backup_import().is_none());
        assert_eq!(app.revision(), 0);
    }

    #[test]
    fn invalid_backup_json_is_rejected_without_a_pending_preview() {
        let existing = nota_core::Note::new("Local".to_string(), "Keep me".to_string());
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
            ThemePreference::Light,
            None,
        );

        app.apply(AppMsg::ImportBackupJson("{not json".to_string()));

        assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
        assert!(app.pending_backup_import().is_none());
        assert_eq!(
            app.notification
                .as_ref()
                .map(|notification| notification.tone),
            Some(NotificationTone::Error)
        );
        assert_eq!(app.revision(), 0);
    }
}
