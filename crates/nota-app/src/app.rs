use chrono::{DateTime, Utc};
use nota_core::NoteWorkspace;
use nota_core::backup::{BackupHealth, BackupHealthRecord, assess_backup_health};
use nota_core::editor_view::EditorViewMode;
use nota_core::exchange::{
    CommitPlan, ConfirmCopy, ExchangeError, ImportDestination, StagedImport, confirm_copy,
    prepare_commit,
};
use nota_core::identity::Notebook;
use nota_core::layout::LayoutFrame;
use nota_core::note_list_interaction::{NoteListInteraction, NoteListRenderModel};
use nota_core::tag_rules::{TagSuggestion, parse_tags_input, suggest_existing_tags};
use nota_core::transition::{ThemePreference, TransitionError, import_desktop_transition};
use uuid::Uuid;

use crate::storage::{CollectionEnvelope, Preferences};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedImport {
    Merged,
    Replaced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalNotification {
    pub message: String,
    pub tone: NotificationTone,
}

#[derive(Debug)]
pub struct AppModel {
    pub workspace: NoteWorkspace,
    pub note_list: NoteListInteraction,
    pub theme: ThemePreference,
    pub view_mode: EditorViewMode,
    pub width_px: f64,
    pub layout: LayoutFrame,
    pub note_list_visible: bool,
    pub save_status: SaveStatus,
    pub notification: Option<GlobalNotification>,
    pub backup_health: Option<BackupHealthRecord>,
    pending_import: Option<StagedImport>,
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
        let view_mode = if collection.notes.is_empty() {
            EditorViewMode::Write
        } else {
            EditorViewMode::Preview
        };
        let width_px = Preferences::default().window_width as f64;
        Self {
            workspace: open_workspace(collection.notes, collection.recently_deleted_notes),
            note_list: NoteListInteraction::default(),
            theme,
            view_mode,
            width_px,
            layout: LayoutFrame::from_width(width_px, view_mode)
                .expect("the default window width is positive"),
            note_list_visible: false,
            save_status: SaveStatus::Saved,
            notification: None,
            backup_health,
            pending_import: None,
            storage_recovery: false,
            notification_generation: 0,
            editing_tags: false,
            revision: 0,
        }
    }

    fn touch(&mut self, changed: bool) -> bool {
        if changed {
            self.revision = self.revision.saturating_add(1);
            self.save_status = SaveStatus::Saving;
        }
        changed
    }

    pub fn quick_capture(&mut self) -> bool {
        self.workspace.create_note();
        self.note_list = NoteListInteraction::default();
        self.note_list_visible = false;
        self.view_mode = EditorViewMode::Write;
        if let Some(layout) = LayoutFrame::from_width(self.width_px, self.view_mode) {
            self.layout = layout;
        }
        self.touch(true)
    }

    pub fn select_note(&mut self, id: Uuid) -> bool {
        if self.workspace.select_note(id) {
            self.note_list_visible = false;
        }
        self.touch(false)
    }

    pub fn update_title(&mut self, title: String) -> bool {
        let changed = self.workspace.update_selected_title(title);
        self.touch(changed)
    }

    pub fn update_content(&mut self, content: String) -> bool {
        let changed = self.workspace.update_selected_content(content);
        self.touch(changed)
    }

    pub fn update_tags(&mut self, tags: String) -> bool {
        let changed = self.workspace.update_selected_tags(parse_tags_input(&tags));
        self.touch(changed)
    }

    pub fn apply_tag_cleanup(&mut self) -> bool {
        let plan = self.workspace.tag_cleanup_plan();
        let changed = self.workspace.apply_tag_cleanup(&plan);
        if changed {
            self.set_notification("Tags cleaned up", NotificationTone::Success);
        }
        self.touch(changed)
    }

    pub fn edit_search(&mut self, search: String) -> bool {
        self.note_list.edit_search(search);
        self.touch(false)
    }

    pub fn commit_search(&mut self) -> bool {
        self.note_list.commit_search();
        self.touch(false)
    }

    pub fn select_tag(&mut self, tag: String) -> bool {
        self.note_list.select_tag(tag);
        self.note_list_visible = true;
        self.touch(false)
    }

    pub fn clear_tag(&mut self) -> bool {
        self.note_list.clear_tag();
        self.touch(false)
    }

    pub fn set_pinned_filter(&mut self, pinned: bool) -> bool {
        self.note_list.set_pinned_only(pinned);
        self.note_list_visible = true;
        self.touch(false)
    }

    pub fn toggle_pin(&mut self, id: Uuid) -> bool {
        let changed = self.workspace.toggle_pin(id);
        self.touch(changed)
    }

    pub fn request_delete(&mut self, id: Uuid) -> bool {
        self.workspace.request_delete(id);
        self.touch(false)
    }

    pub fn cancel_delete(&mut self) -> bool {
        self.workspace.cancel_delete();
        self.touch(false)
    }

    pub fn confirm_delete(&mut self) -> bool {
        let changed = self.workspace.confirm_delete();
        self.touch(changed)
    }

    pub fn restore_recently_deleted(&mut self, id: Uuid) -> bool {
        let changed = self.workspace.restore_recently_deleted(id);
        self.touch(changed)
    }

    pub fn permanently_delete(&mut self, id: Uuid) -> bool {
        let changed = self.workspace.permanently_clear_recently_deleted(id);
        self.touch(changed)
    }

    pub fn request_clear_all(&mut self) -> bool {
        self.workspace.request_clear_all_recently_deleted()
    }

    pub fn cancel_clear_all(&mut self) -> bool {
        self.workspace.cancel_clear_all_recently_deleted();
        self.touch(false)
    }

    pub fn confirm_clear_all(&mut self) -> bool {
        let changed = self.workspace.confirm_clear_all_recently_deleted();
        self.touch(changed)
    }

    pub fn set_view_mode(&mut self, mode: EditorViewMode) -> bool {
        self.view_mode = mode;
        if let Some(layout) = LayoutFrame::from_width(self.width_px, mode) {
            self.layout = layout;
        }
        self.touch(false)
    }

    pub fn resize(&mut self, width: f64) -> bool {
        if let Some(layout) = LayoutFrame::from_width(width, self.view_mode) {
            self.width_px = width;
            self.layout = layout;
        }
        self.touch(false)
    }

    pub fn toggle_navigation(&mut self) -> bool {
        self.note_list_visible = !self.note_list_visible;
        self.touch(false)
    }

    pub fn note_saved(&mut self, revision: u64) -> bool {
        if revision == self.revision {
            self.save_status = SaveStatus::Saved;
        }
        self.touch(false)
    }

    pub fn note_save_failed(&mut self, error: String) -> bool {
        self.save_status = SaveStatus::Failed;
        self.set_notification(error, NotificationTone::Error);
        self.touch(false)
    }

    pub fn backup_exported(&mut self, exported_at: DateTime<Utc>) -> bool {
        self.backup_health = Some(BackupHealthRecord {
            last_successful_export_at: exported_at,
        });
        self.set_notification("Backup exported", NotificationTone::Success);
        self.touch(false)
    }

    pub fn toggle_theme(&mut self) -> ThemePreference {
        self.theme = match self.theme {
            ThemePreference::Dark => ThemePreference::Light,
            ThemePreference::Light | ThemePreference::System => ThemePreference::Dark,
        };
        self.theme
    }

    pub fn notify_success(&mut self, message: impl Into<String>) -> bool {
        self.set_notification(message, NotificationTone::Success);
        self.touch(false)
    }

    pub fn notify_failure(&mut self, message: impl Into<String>) -> bool {
        self.set_notification(message, NotificationTone::Error);
        self.touch(false)
    }

    pub fn dismiss_notification(&mut self, generation: u64) -> bool {
        if generation == self.notification_generation {
            self.notification = None;
        }
        self.touch(false)
    }

    pub fn start_edit_tags(&mut self) -> bool {
        self.editing_tags = true;
        self.touch(false)
    }

    pub fn finish_edit_tags(&mut self) -> bool {
        self.editing_tags = false;
        self.touch(false)
    }

    pub fn accept_tag_suggestion(&mut self, input: String) -> bool {
        let Some(suggestion) = self.tag_suggestions(&input).into_iter().next() else {
            return self.touch(false);
        };
        let changed = self
            .workspace
            .update_selected_tags(parse_tags_input(&suggestion.completed_input));
        self.touch(changed)
    }

    pub fn cancel_import(&mut self) -> bool {
        self.pending_import = None;
        self.set_notification("Backup import cancelled", NotificationTone::Progress);
        self.touch(false)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn pending_import(&self) -> Option<ConfirmCopy> {
        self.pending_import
            .as_ref()
            .map(|staged| confirm_copy(staged, Utc::now()))
    }

    pub(crate) fn staged_import(&self) -> Option<&StagedImport> {
        self.pending_import.as_ref()
    }

    pub fn stage_import(&mut self, staged: StagedImport) {
        self.pending_import = Some(staged);
        self.set_notification("Backup ready", NotificationTone::Success);
    }

    pub(crate) fn clear_pending_import(&mut self) {
        self.pending_import = None;
    }

    pub fn commit_staged_import(&mut self) -> Result<Option<AppliedImport>, ExchangeError> {
        let Some(staged) = self.pending_import.clone() else {
            return Ok(None);
        };
        let plan = prepare_commit(&staged, self.import_destination())?;
        let applied = match &plan {
            CommitPlan::MergeNotes { flat_json } => {
                self.import_backup(flat_json)
                    .map_err(ExchangeError::Backup)?;
                self.revision = self.revision.saturating_add(1);
                self.save_status = SaveStatus::Saving;
                AppliedImport::Merged
            }
            CommitPlan::ReplaceNotebook { transition_json } => {
                self.import_transition(transition_json)
                    .map_err(ExchangeError::Transition)?;
                AppliedImport::Replaced
            }
        };
        self.pending_import = None;
        self.storage_recovery = false;
        Ok(Some(applied))
    }

    fn import_destination(&self) -> ImportDestination<'_> {
        ImportDestination {
            notes: self.workspace.notes(),
            recently_deleted: self.workspace.recently_deleted_notes(),
            storage_recovery: self.storage_recovery,
        }
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
        self.workspace = open_workspace(collection.notes, collection.recently_deleted_notes);
        self.save_status = SaveStatus::Saved;
        self.notification = None;
    }

    pub fn import_backup(&mut self, json: &str) -> Result<(), nota_core::backup::BackupError> {
        let imported = self.workspace.import_flat_collection_backup(json)?;
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
        self.workspace = open_workspace(restored.notes, restored.recently_deleted_notes);
        self.theme = restored.theme;
        self.backup_health = restored.backup_health;
        self.mark_external_change("Desktop transition restored");
        Ok(())
    }

    fn mark_external_change(&mut self, message: &str) {
        self.revision = self.revision.saturating_add(1);
        self.save_status = SaveStatus::Saving;
        self.set_notification(message, NotificationTone::Success);
    }
}

fn open_workspace(active: Vec<nota_core::Note>, deleted: Vec<nota_core::Note>) -> NoteWorkspace {
    NoteWorkspace::try_open(
        Notebook::try_new(active, deleted)
            .expect("collection identities were checked before opening"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use nota_core::transition::export_desktop_transition;

    #[test]
    fn quick_capture_edit_search_delete_restore_and_clear_follow_messages() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert!(app.quick_capture());
        let id = app.workspace.selected_id().unwrap();
        assert!(app.update_title("Roadmap".to_string()));
        assert!(app.update_content("Ship native app".to_string()));
        assert!(app.update_tags("Work, Rust".to_string()));

        app.edit_search("native".to_string());
        app.commit_search();
        assert_eq!(app.note_list_render_model().projection.rows.len(), 1);

        app.request_delete(id);
        assert!(app.confirm_delete());
        assert!(app.workspace.notes().is_empty());
        assert_eq!(app.workspace.recently_deleted_notes().len(), 1);

        assert!(app.restore_recently_deleted(id));
        app.request_delete(id);
        app.confirm_delete();
        app.request_clear_all();
        assert!(app.confirm_clear_all());
        assert!(app.workspace.recently_deleted_notes().is_empty());
    }

    #[test]
    fn resize_ignores_non_positive_widths_and_preserves_the_closed_drawer() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        assert!(!app.layout.drawer_fills_width);
        assert!(!app.note_list_visible);

        app.resize(0.0);
        assert!(!app.layout.drawer_fills_width);
        assert!(!app.note_list_visible);

        app.resize(-1.0);
        assert!(!app.layout.drawer_fills_width);
    }

    #[test]
    fn split_view_survives_resize_and_can_be_selected_on_a_narrow_window() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        app.set_view_mode(EditorViewMode::Split);
        assert_eq!(app.view_mode, EditorViewMode::Split);

        app.resize(500.0);

        assert!(app.layout.drawer_fills_width);
        assert_eq!(app.view_mode, EditorViewMode::Split);
        app.set_view_mode(EditorViewMode::Write);
        app.set_view_mode(EditorViewMode::Split);
        assert_eq!(app.view_mode, EditorViewMode::Split);
        app.resize(1200.0);
        assert!(!app.layout.drawer_fills_width);
        assert_eq!(app.view_mode, EditorViewMode::Split);
    }

    #[test]
    fn drawer_toggles_at_every_width_and_selection_closes_it_without_revising_notes() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        app.quick_capture();
        let id = app.workspace.selected_id().unwrap();
        let revision = app.revision();
        for width in [500.0, 1200.0] {
            app.resize(width);
            assert!(!app.note_list_visible);
            assert!(!app.toggle_navigation());
            assert!(app.note_list_visible);
            app.resize(width + 1.0);
            assert!(app.note_list_visible);
            app.select_note(Uuid::new_v4());
            assert!(app.note_list_visible);
            app.select_note(id);
            assert!(!app.note_list_visible);
        }
        assert_eq!(app.revision(), revision);
    }

    #[test]
    fn existing_notebook_opens_in_preview_and_capture_opens_a_new_note_for_writing() {
        let note = nota_core::Note::new("Existing".to_string(), "Body".to_string());
        let mut app = AppModel::new(
            CollectionEnvelope::new(vec![note.clone()], Vec::new()),
            ThemePreference::Light,
            None,
        );
        assert_eq!(app.workspace.selected_id(), Some(note.id));
        assert_eq!(app.view_mode, EditorViewMode::Preview);
        assert!(!app.note_list_visible);
        app.toggle_navigation();
        assert!(app.quick_capture());
        assert_ne!(app.workspace.selected_id(), Some(note.id));
        assert_eq!(app.view_mode, EditorViewMode::Write);
        assert!(!app.note_list_visible);
        assert_eq!(
            app.workspace.focus_intent(),
            nota_core::note_workspace::FocusIntent::NoteTitle
        );
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
        app.select_note(selected.id);

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
        app.select_note(selected.id);

        assert!(app.accept_tag_suggestion("Work, r".to_string()));
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
        app.select_note(older_id);
        let before: Vec<_> = app
            .note_list_render_model()
            .projection
            .rows
            .iter()
            .map(|row| row.id)
            .collect();

        app.select_note(newer_id);

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

        app.toggle_theme();

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

    fn stage_json(app: &mut AppModel, json: &str) {
        let staged = nota_core::exchange::stage_import(
            json,
            nota_core::exchange::ImportDestination {
                notes: app.workspace.notes(),
                recently_deleted: app.workspace.recently_deleted_notes(),
                storage_recovery: app.is_in_storage_recovery(),
            },
        )
        .unwrap();
        app.stage_import(staged);
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

        stage_json(&mut app, &json);

        assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
        let pending = app.pending_import().expect("import copy should be pending");
        assert_eq!(pending.title, "Import these notes?");
        assert_eq!(pending.action, "Import");
        assert_eq!(
            pending.body,
            "Notes in this file: 1\n\nNotes to add: 1\nNotes to replace: 0"
        );
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

        stage_json(&mut app, &json);

        let pending = app.pending_import().expect("import copy should be pending");
        assert_eq!(
            pending.body,
            "Notes in this file: 1\n\nNotes to add: 0\nNotes to replace: 1"
        );
        assert_eq!(app.workspace.notes()[0].title, "Original");
        assert_eq!(
            app.commit_staged_import().unwrap(),
            Some(AppliedImport::Merged)
        );
        assert_eq!(app.workspace.notes()[0].title, "Replaced");
    }

    #[test]
    fn storage_recovery_flag_is_recorded_for_the_session_gate() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert!(!app.is_in_storage_recovery());
        app.set_storage_recovery(true);
        assert!(app.is_in_storage_recovery());
        assert!(app.quick_capture());
        assert_eq!(app.workspace.notes().len(), 1);
        app.set_storage_recovery(false);
        assert!(!app.is_in_storage_recovery());
    }

    #[test]
    fn quick_capture_requests_note_title_focus() {
        use nota_core::note_workspace::FocusIntent;

        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert!(app.quick_capture());
        assert_eq!(app.workspace.focus_intent(), FocusIntent::NoteTitle);
    }

    #[test]
    fn tag_editing_is_opt_in_and_does_not_revise_the_collection() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.start_edit_tags();
        assert!(app.is_editing_tags());
        assert_eq!(app.revision(), 0);
        app.finish_edit_tags();
        assert!(!app.is_editing_tags());
    }

    #[test]
    fn storage_recovery_allows_backup_import_preview_and_merge() {
        use nota_core::backup::export_flat_collection_backup;

        let imported = nota_core::Note::new("Recovered".to_string(), "From Backup".to_string());
        let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.set_storage_recovery(true);

        stage_json(&mut app, &json);
        assert!(app.pending_import().is_some());
        assert_eq!(
            app.commit_staged_import().unwrap(),
            Some(AppliedImport::Merged)
        );
        assert!(!app.is_in_storage_recovery());
        assert_eq!(app.workspace.notes(), std::slice::from_ref(&imported));
    }

    #[test]
    fn global_notifications_can_be_dismissed_without_mutating_notes() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        app.notify_success("Backup exported".to_string());
        assert!(app.notification.is_some());
        let generation = app.notification_generation();
        app.dismiss_notification(generation);
        assert!(app.notification.is_none());
        assert_eq!(app.revision(), 0);
    }

    #[test]
    fn backup_health_sentences_cover_missing_recent_and_stale() {
        use chrono::TimeZone;

        let missing = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        let now = chrono::Utc.with_ymd_and_hms(2026, 9, 4, 12, 0, 0).unwrap();
        assert_eq!(
            missing.backup_health_status(now).sentences().label,
            "No backup yet"
        );
        assert_eq!(
            missing.backup_health_status(now).sentences().detail,
            "No backup yet"
        );

        let recent = AppModel::new(
            CollectionEnvelope::empty(),
            ThemePreference::Light,
            Some(BackupHealthRecord {
                last_successful_export_at: now,
            }),
        );
        assert_eq!(
            recent.backup_health_status(now).sentences().label,
            "Up to date"
        );

        let stale = AppModel::new(
            CollectionEnvelope::empty(),
            ThemePreference::Light,
            Some(BackupHealthRecord {
                last_successful_export_at: now - chrono::Duration::days(15),
            }),
        );
        assert_eq!(
            stale.backup_health_status(now).sentences().label,
            "Backup stale"
        );
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
        app.edit_search("zzzz".to_string());
        assert_eq!(
            app.note_list_render_model().display_state,
            NoteListDisplayState::Rows
        );
        app.commit_search();
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

        app.select_tag("Work".to_string());
        let filtered = app.note_list_render_model();
        assert_eq!(filtered.projection.rows.len(), 1);
        assert_eq!(app.note_list.active_tag(), Some("Work"));

        app.clear_tag();
        assert!(app.note_list.active_tag().is_none());
        assert_eq!(app.note_list_render_model().projection.rows.len(), 2);

        app.edit_search("unique".to_string());
        app.commit_search();
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

        stage_json(&mut app, &json);
        assert_eq!(
            app.commit_staged_import().unwrap(),
            Some(AppliedImport::Merged)
        );

        assert_eq!(app.workspace.notes().len(), 2);
        assert!(app.pending_import().is_none());
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

        stage_json(&mut app, &json);
        app.cancel_import();

        assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
        assert!(app.pending_import().is_none());
        assert_eq!(app.revision(), 0);
    }
}
