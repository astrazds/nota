use crate::storage::{
    SaveStatus, has_quarantined_corrupt_payloads, quarantine_corrupt_payloads,
    save_backup_health_record, save_note_collection,
};
use chrono::Utc;
use leptos::prelude::*;
use nota_core::backup::{
    BackupError, BackupHealth, BackupHealthRecord, assess_backup_health,
    export_flat_collection_backup,
};
use nota_core::editor_view::EditorViewMode;
use nota_core::model::Note;
use nota_core::note_discovery::{NoteListItem, SelectedNoteVisibility};
use nota_core::note_list_interaction::{
    NoteActionControls, NoteListCommand, NoteListInteraction, NoteListRenderModel,
};
use nota_core::note_workspace::{FocusIntent, NoteWorkspace, WorkspaceDisplayState};
use nota_core::responsive_navigation::{
    NoteListPersistence, ResponsiveNavigation, StoredNoteListState, ViewportClass,
    normalize_view_mode,
};
use nota_core::storage_recovery::{
    StorageRecoveryChoice, StorageRecoveryState, resolve_storage_recovery,
};
use nota_core::tag_rules::{
    TagCleanupPlan, TagSuggestion, collect_note_tags, suggest_existing_tags,
};
use nota_core::transition::{ThemePreference, TransitionError, export_desktop_transition};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct AppState {
    pub workspace: RwSignal<NoteWorkspace>,
    pub notes_save_revision: RwSignal<u64>,
    pub is_dark_mode: RwSignal<bool>,
    pub viewport_class: RwSignal<ViewportClass>,
    pub is_sidebar_open: RwSignal<bool>,
    pub note_list_interaction: RwSignal<NoteListInteraction>,
    pub editor_view_mode: RwSignal<EditorViewMode>,
    pub save_status: RwSignal<SaveStatus>,
    pub backup_health_record: RwSignal<Option<BackupHealthRecord>>,
    pub notification: RwSignal<Option<GlobalNotification>>,
    pub notification_sequence: RwSignal<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalNotification {
    pub id: u64,
    pub message: String,
    pub tone: NotificationTone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationTone {
    Progress,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct AppRuntimeStartup {
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
    pub is_dark_mode: bool,
    pub viewport_class: ViewportClass,
    pub stored_note_list_state: StoredNoteListState,
    pub backup_health_record: Option<BackupHealthRecord>,
    pub storage_recovery: Option<StorageRecoveryState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePersistenceSnapshot {
    pub is_dark_mode: bool,
    pub viewport_class: ViewportClass,
    pub note_list_state_to_persist: Option<StoredNoteListState>,
    pub notes_save_revision: u64,
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
    pub editor_view_mode: EditorViewMode,
    pub backup_health_record: Option<BackupHealthRecord>,
    pub save_status: SaveStatus,
}

impl AppState {
    pub fn from_startup(startup: AppRuntimeStartup) -> Self {
        let initial_navigation =
            ResponsiveNavigation::initial(startup.viewport_class, startup.stored_note_list_state);

        Self {
            workspace: RwSignal::new(if let Some(storage_recovery) = startup.storage_recovery {
                NoteWorkspace::new_with_storage_recovery(storage_recovery)
            } else {
                NoteWorkspace::new_with_recently_deleted(
                    startup.notes,
                    startup.recently_deleted_notes,
                )
            }),
            notes_save_revision: RwSignal::new(0),
            is_dark_mode: RwSignal::new(startup.is_dark_mode),
            viewport_class: RwSignal::new(startup.viewport_class),
            is_sidebar_open: RwSignal::new(initial_navigation.is_note_list_visible()),
            note_list_interaction: RwSignal::new(NoteListInteraction::default()),
            editor_view_mode: RwSignal::new(EditorViewMode::Write),
            save_status: RwSignal::new(SaveStatus::Saved),
            backup_health_record: RwSignal::new(startup.backup_health_record),
            notification: RwSignal::new(None),
            notification_sequence: RwSignal::new(0),
        }
    }

    pub fn persistence_snapshot(self) -> RuntimePersistenceSnapshot {
        self.persistence_snapshot_with_revision(self.notes_save_revision.get_untracked())
    }

    pub fn tracked_notes_persistence_snapshot(self) -> RuntimePersistenceSnapshot {
        let revision = self.notes_save_revision.get();
        self.persistence_snapshot_with_revision(revision)
    }

    pub fn note_list_state_to_persist(self) -> Option<StoredNoteListState> {
        let navigation =
            ResponsiveNavigation::current(self.viewport_class.get(), self.is_sidebar_open.get());
        match navigation.persistence() {
            NoteListPersistence::Persist(stored_state) => Some(stored_state),
            NoteListPersistence::Skip => None,
        }
    }

    pub fn reclassify_viewport(self, next_viewport_class: ViewportClass) -> bool {
        if self.viewport_class.get_untracked() == next_viewport_class {
            return false;
        }

        let mut navigation = self.responsive_navigation_untracked();
        navigation.reclassify_viewport(next_viewport_class);

        self.viewport_class.set(next_viewport_class);
        self.is_sidebar_open.set(navigation.is_note_list_visible());
        self.editor_view_mode.update(|view_mode| {
            *view_mode = normalize_view_mode(next_viewport_class, *view_mode);
        });
        true
    }

    fn persistence_snapshot_with_revision(
        self,
        notes_save_revision: u64,
    ) -> RuntimePersistenceSnapshot {
        RuntimePersistenceSnapshot {
            is_dark_mode: self.is_dark_mode.get_untracked(),
            viewport_class: self.viewport_class.get_untracked(),
            note_list_state_to_persist: self.note_list_state_to_persist_untracked(),
            notes_save_revision,
            notes: self.notes_untracked(),
            recently_deleted_notes: self.recently_deleted_notes_untracked(),
            editor_view_mode: self.editor_view_mode.get_untracked(),
            backup_health_record: self.backup_health_record.get_untracked(),
            save_status: self.save_status.get_untracked(),
        }
    }

    fn note_list_state_to_persist_untracked(self) -> Option<StoredNoteListState> {
        let navigation = ResponsiveNavigation::current(
            self.viewport_class.get_untracked(),
            self.is_sidebar_open.get_untracked(),
        );
        match navigation.persistence() {
            NoteListPersistence::Persist(stored_state) => Some(stored_state),
            NoteListPersistence::Skip => None,
        }
    }
}

impl AppState {
    pub fn notes_untracked(self) -> Vec<Note> {
        self.workspace.get_untracked().notes().to_vec()
    }

    pub fn recently_deleted_notes(self) -> Vec<Note> {
        self.workspace.get().recently_deleted_notes().to_vec()
    }

    pub fn recently_deleted_notes_untracked(self) -> Vec<Note> {
        self.workspace
            .get_untracked()
            .recently_deleted_notes()
            .to_vec()
    }

    pub fn selected_id(self) -> Option<Uuid> {
        self.workspace.get().selected_id()
    }

    pub fn selected_note(self) -> Option<Note> {
        self.workspace.get().selected_note()
    }

    pub fn workspace_display_state(self) -> WorkspaceDisplayState {
        self.workspace.get().display_state()
    }

    pub fn storage_recovery(self) -> Option<StorageRecoveryState> {
        self.workspace.get().storage_recovery().cloned()
    }

    pub fn has_previous_snapshot(self) -> bool {
        self.storage_recovery()
            .is_some_and(|recovery| recovery.previous_snapshot.is_some())
    }

    pub fn note_list_render_model(self) -> NoteListRenderModel {
        let workspace = self.workspace.get();
        self.note_list_interaction
            .get()
            .render_model(workspace.notes(), workspace.selected_id())
    }

    pub fn selected_note_is_hidden_by_filter(self) -> bool {
        self.note_list_render_model()
            .projection
            .selected_note_visibility
            == SelectedNoteVisibility::HiddenByFilter
    }

    pub fn note_search_input(self) -> String {
        self.note_list_interaction
            .get_untracked()
            .search_input()
            .to_string()
    }

    pub fn edit_note_search(self, input: String) {
        self.note_list_interaction
            .update(|interaction| interaction.edit_search(input));
    }

    pub fn commit_note_search(self) {
        self.note_list_interaction
            .update(NoteListInteraction::commit_search);
    }

    pub fn active_tag(self) -> Option<String> {
        self.note_list_interaction
            .get()
            .active_tag()
            .map(str::to_string)
    }

    pub fn select_active_tag(self, tag: String) {
        self.note_list_interaction
            .update(|interaction| interaction.select_tag(tag));
    }

    pub fn clear_active_tag(self) {
        self.note_list_interaction
            .update(NoteListInteraction::clear_tag);
    }

    pub fn note_actions(self, row: &NoteListItem) -> NoteActionControls {
        self.note_list_interaction.get_untracked().note_actions(row)
    }

    pub fn select_note_list_row(self, id: Uuid) {
        self.select_note(id);
    }

    pub fn apply_note_list_command(self, command: NoteListCommand) {
        match command {
            NoteListCommand::SelectNote(id) => self.select_note(id),
            NoteListCommand::TogglePin(id) => self.toggle_note_pin(id),
            NoteListCommand::RequestDelete(id) => self.request_delete_note(id),
        }
    }

    pub fn available_tags(self) -> Vec<String> {
        collect_note_tags(self.workspace.get().notes())
    }

    pub fn tag_suggestions(self, input: &str) -> Vec<TagSuggestion> {
        let workspace = self.workspace.get();
        let selected_note = workspace.selected_note();
        suggest_existing_tags(workspace.notes(), selected_note.as_ref(), input)
    }

    pub fn create_note(self) {
        self.quick_capture_note();
    }

    pub fn quick_capture_note(self) {
        self.workspace.update(NoteWorkspace::create_note);
        self.note_selected();
        self.mark_notes_changed();
    }

    pub fn select_note(self, id: Uuid) {
        let selected = self
            .workspace
            .try_update(|workspace| workspace.select_note(id))
            .unwrap_or(false);
        if selected {
            self.note_selected();
        }
    }

    pub fn request_delete_note(self, id: Uuid) {
        self.workspace.update(|workspace| {
            workspace.request_delete(id);
        });
    }

    pub fn is_delete_confirmation_open(self) -> bool {
        self.workspace.get().is_delete_confirmation_open()
    }

    pub fn delete_confirmation_title(self) -> Option<String> {
        self.workspace
            .get()
            .delete_confirmation_title()
            .map(str::to_string)
    }

    pub fn cancel_delete_note(self) {
        self.workspace.update(NoteWorkspace::cancel_delete);
    }

    pub fn confirm_delete_selected_note(self) {
        let deleted = self
            .workspace
            .try_update(NoteWorkspace::confirm_delete)
            .unwrap_or(false);
        if deleted {
            self.mark_notes_changed();
        }
    }

    pub fn restore_recently_deleted_note(self, id: Uuid) {
        let restored = self
            .workspace
            .try_update(|workspace| workspace.restore_recently_deleted(id))
            .unwrap_or(false);
        if restored {
            self.note_selected();
            self.mark_notes_changed();
        }
    }

    pub fn permanently_clear_recently_deleted_note(self, id: Uuid) {
        let cleared = self
            .workspace
            .try_update(|workspace| workspace.permanently_clear_recently_deleted(id))
            .unwrap_or(false);
        if cleared {
            self.mark_notes_changed();
        }
    }

    pub fn request_clear_all_recently_deleted(self) {
        self.workspace.update(|workspace| {
            workspace.request_clear_all_recently_deleted();
        });
    }

    pub fn clear_all_recently_deleted_confirmation_count(self) -> Option<usize> {
        self.workspace
            .get()
            .clear_all_recently_deleted_confirmation_count()
    }

    pub fn cancel_clear_all_recently_deleted(self) {
        self.workspace
            .update(NoteWorkspace::cancel_clear_all_recently_deleted);
    }

    pub fn confirm_clear_all_recently_deleted_notes(self) {
        let cleared = self
            .workspace
            .try_update(NoteWorkspace::confirm_clear_all_recently_deleted)
            .unwrap_or(false);
        if cleared {
            self.mark_notes_changed();
            self.show_notification("Recently Deleted cleared", NotificationTone::Success);
        }
    }

    pub fn restore_previous_snapshot(self) {
        let restored = self
            .workspace
            .try_update(NoteWorkspace::restore_previous_snapshot)
            .unwrap_or(false);
        if restored {
            self.note_selected();
            self.mark_notes_changed();
            save_note_collection(
                &self.notes_untracked(),
                &self.recently_deleted_notes_untracked(),
            );
            self.show_notification("Previous snapshot restored", NotificationTone::Success);
        }
    }

    pub fn start_empty_after_storage_recovery(self) {
        let payload_to_quarantine = self.storage_recovery().and_then(|recovery| {
            resolve_storage_recovery(&recovery, StorageRecoveryChoice::StartEmpty)
                .map(|resolution| resolution.corrupt_payloads_to_quarantine)
        });

        let started_empty = self
            .workspace
            .try_update(NoteWorkspace::start_empty_after_storage_recovery)
            .unwrap_or(false);
        if started_empty {
            if let Some(payload) = payload_to_quarantine {
                quarantine_corrupt_payloads(&payload);
            }
            self.mark_notes_changed();
            save_note_collection(&[], &[]);
            self.show_notification("Started empty collection", NotificationTone::Success);
        }
    }

    pub fn has_quarantined_corrupt_payloads(self) -> bool {
        has_quarantined_corrupt_payloads()
    }

    pub fn update_selected_title(self, title: String) {
        let updated = self
            .workspace
            .try_update(|workspace| workspace.update_selected_title(title.clone()))
            .unwrap_or(false);
        if updated {
            self.mark_notes_changed();
        }
    }

    pub fn update_selected_content(self, content: String) {
        let updated = self
            .workspace
            .try_update(|workspace| workspace.update_selected_content(content.clone()))
            .unwrap_or(false);
        if updated {
            self.mark_notes_changed();
        }
    }

    pub fn update_selected_tags(self, tags: Vec<String>) {
        let updated = self
            .workspace
            .try_update(|workspace| workspace.update_selected_tags(tags.clone()))
            .unwrap_or(false);
        if updated {
            self.mark_notes_changed();
        }
    }

    pub fn tag_cleanup_plan(self) -> TagCleanupPlan {
        self.workspace.get().tag_cleanup_plan()
    }

    pub fn apply_tag_cleanup(self, plan: &TagCleanupPlan) {
        let updated = self
            .workspace
            .try_update(|workspace| workspace.apply_tag_cleanup(plan))
            .unwrap_or(false);
        if updated {
            self.mark_notes_changed();
        }
    }

    pub fn export_backup_json(self) -> Result<String, BackupError> {
        export_flat_collection_backup(self.workspace.get_untracked().notes())
    }

    pub fn export_desktop_transition_json(self) -> Result<String, TransitionError> {
        let workspace = self.workspace.get_untracked();
        export_desktop_transition(
            workspace.notes(),
            workspace.recently_deleted_notes(),
            if self.is_dark_mode.get_untracked() {
                ThemePreference::Dark
            } else {
                ThemePreference::Light
            },
            self.backup_health_record.get_untracked(),
        )
    }

    pub fn backup_health(self) -> BackupHealth {
        assess_backup_health(self.backup_health_record.get(), Utc::now())
    }

    pub fn backup_health_summary(self) -> String {
        match self.backup_health() {
            BackupHealth::Missing => "No backup yet. Export now".to_string(),
            BackupHealth::Recent {
                last_successful_export_at,
            } => format!("Backed up {}", last_successful_export_at.format("%d/%m/%Y")),
            BackupHealth::Stale {
                last_successful_export_at,
            } => format!(
                "Backup stale {}. Export now",
                last_successful_export_at.format("%d/%m/%Y")
            ),
        }
    }

    pub fn record_backup_exported_at(self, last_successful_export_at: chrono::DateTime<Utc>) {
        let record = BackupHealthRecord {
            last_successful_export_at,
        };
        self.backup_health_record.set(Some(record));
        save_backup_health_record(record);
    }

    pub fn show_notification(self, message: impl Into<String>, tone: NotificationTone) {
        let mut id = 0;
        self.notification_sequence.update(|sequence| {
            *sequence = sequence.wrapping_add(1);
            id = *sequence;
        });
        self.notification.set(Some(GlobalNotification {
            id,
            message: message.into(),
            tone,
        }));
    }

    pub fn show_save_notification(self, message: impl Into<String>, tone: NotificationTone) {
        let can_replace = self
            .notification
            .get_untracked()
            .is_none_or(|notification| is_save_notification(&notification.message));
        if can_replace {
            self.show_notification(message, tone);
        }
    }

    pub fn clear_notification(self, id: u64) {
        let should_clear = self
            .notification
            .get_untracked()
            .is_some_and(|notification| notification.id == id);
        if should_clear {
            self.notification.set(None);
        }
    }

    pub fn import_backup_json(self, backup_json: &str) -> Result<(), BackupError> {
        self.workspace
            .try_update(|workspace| workspace.import_flat_collection_backup(backup_json))
            .unwrap_or_else(|| {
                Err(BackupError::UnsupportedKind(
                    "missing workspace".to_string(),
                ))
            })?;
        self.mark_notes_changed();
        save_note_collection(
            &self.notes_untracked(),
            &self.recently_deleted_notes_untracked(),
        );
        Ok(())
    }

    pub fn toggle_note_pin(self, id: Uuid) {
        let updated = self
            .workspace
            .try_update(|workspace| workspace.toggle_pin(id))
            .unwrap_or(false);
        if updated {
            self.mark_notes_changed();
        }
    }

    pub fn take_focus_intent(self) -> FocusIntent {
        self.workspace
            .try_update(NoteWorkspace::take_focus_intent)
            .unwrap_or_default()
    }

    pub fn focus_intent(self) -> FocusIntent {
        self.workspace.get().focus_intent()
    }

    fn mark_notes_changed(self) {
        self.notes_save_revision
            .update(|revision| *revision = revision.wrapping_add(1));
    }

    pub fn toggle_dark_mode(self) {
        self.is_dark_mode.update(|enabled| *enabled = !*enabled);
    }

    pub fn toggle_sidebar(self) {
        let mut navigation = self.responsive_navigation_untracked();
        navigation.toggle_note_list();
        self.is_sidebar_open.set(navigation.is_note_list_visible());
    }

    pub fn set_editor_view_mode(self, view_mode: EditorViewMode) {
        self.editor_view_mode.set(normalize_view_mode(
            self.viewport_class.get_untracked(),
            view_mode,
        ));
    }

    fn note_selected(self) {
        let mut navigation = self.responsive_navigation_untracked();
        navigation.note_selected();
        self.is_sidebar_open.set(navigation.is_note_list_visible());
    }

    fn responsive_navigation_untracked(self) -> ResponsiveNavigation {
        ResponsiveNavigation::current(
            self.viewport_class.get_untracked(),
            self.is_sidebar_open.get_untracked(),
        )
    }
}

fn is_save_notification(message: &str) -> bool {
    matches!(message, "Saving..." | "Saved")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppRuntimeStartup;
    use nota_core::responsive_navigation::StoredNoteListState;

    fn with_test_state<T>(
        notes: Vec<Note>,
        viewport_class: ViewportClass,
        is_sidebar_open: bool,
        test: impl FnOnce(AppState) -> T,
    ) -> T {
        Owner::new().with(|| {
            let state = AppState::from_startup(AppRuntimeStartup {
                notes,
                recently_deleted_notes: Vec::new(),
                is_dark_mode: false,
                viewport_class,
                stored_note_list_state: StoredNoteListState::from_is_open(is_sidebar_open),
                backup_health_record: None,
                storage_recovery: None,
            });

            test(state)
        })
    }

    fn with_test_state_and_recently_deleted<T>(
        notes: Vec<Note>,
        recently_deleted_notes: Vec<Note>,
        test: impl FnOnce(AppState) -> T,
    ) -> T {
        Owner::new().with(|| {
            let state = AppState::from_startup(AppRuntimeStartup {
                notes,
                recently_deleted_notes,
                is_dark_mode: false,
                viewport_class: ViewportClass::Wide,
                stored_note_list_state: StoredNoteListState::Open,
                backup_health_record: None,
                storage_recovery: None,
            });

            test(state)
        })
    }

    #[test]
    fn note_list_row_selection_selects_the_note_and_updates_responsive_navigation() {
        let first = Note::new("First".to_string(), String::new());
        let second = Note::new("Second".to_string(), String::new());
        let second_id = second.id;

        with_test_state(vec![first, second], ViewportClass::Compact, true, |state| {
            state.select_note_list_row(second_id);

            assert_eq!(state.selected_id(), Some(second_id));
            assert!(!state.is_sidebar_open.get_untracked());
        });
    }

    #[test]
    fn quick_capture_creates_focuses_and_reveals_a_new_note() {
        let existing = Note::new("Existing".to_string(), String::new());
        let existing_id = existing.id;

        with_test_state(vec![existing], ViewportClass::Compact, true, |state| {
            state.quick_capture_note();

            let notes = state.notes_untracked();
            assert_eq!(notes.len(), 2);
            assert_ne!(notes[0].id, existing_id);
            assert_eq!(state.selected_id(), Some(notes[0].id));
            assert_eq!(state.focus_intent(), FocusIntent::NoteTitle);
            assert!(!state.is_sidebar_open.get_untracked());
            assert_eq!(notes[1].id, existing_id);
            assert_eq!(state.notes_save_revision.get_untracked(), 1);
        });
    }

    #[test]
    fn note_list_action_commands_pin_and_request_delete_notes() {
        let note = Note::new("Action target".to_string(), String::new());
        let note_id = note.id;

        with_test_state(vec![note], ViewportClass::Wide, true, |state| {
            let row = state.note_list_render_model().projection.rows.remove(0);
            let actions = state.note_actions(&row);

            state.apply_note_list_command(actions.pin_command);
            assert!(state.notes_untracked()[0].is_pinned);
            assert_eq!(state.notes_save_revision.get_untracked(), 1);

            state.apply_note_list_command(actions.delete_command);
            assert_eq!(state.selected_id(), Some(note_id));
            assert!(state.is_delete_confirmation_open());
            assert_eq!(
                state.delete_confirmation_title().as_deref(),
                Some("Action target")
            );
        });
    }

    #[test]
    fn recently_deleted_restore_and_clear_mark_notes_changed() {
        let note = Note::new("Recoverable".to_string(), String::new());
        let note_id = note.id;

        with_test_state(vec![note.clone()], ViewportClass::Wide, true, |state| {
            state.request_delete_note(note_id);
            state.confirm_delete_selected_note();

            assert!(state.notes_untracked().is_empty());
            assert_eq!(state.recently_deleted_notes_untracked(), vec![note.clone()]);
            assert_eq!(state.notes_save_revision.get_untracked(), 1);

            state.restore_recently_deleted_note(note_id);

            assert_eq!(state.notes_untracked(), vec![note.clone()]);
            assert!(state.recently_deleted_notes_untracked().is_empty());
            assert_eq!(state.selected_id(), Some(note_id));
            assert_eq!(state.notes_save_revision.get_untracked(), 2);

            state.request_delete_note(note_id);
            state.confirm_delete_selected_note();
            state.permanently_clear_recently_deleted_note(note_id);

            assert!(state.recently_deleted_notes_untracked().is_empty());
            assert_eq!(state.notes_save_revision.get_untracked(), 4);
        });
    }

    #[test]
    fn clear_all_recently_deleted_marks_notes_changed_after_confirmation() {
        let active_note = Note::new("Active".to_string(), String::new());
        let first_deleted = Note::new("First deleted".to_string(), String::new());
        let second_deleted = Note::new("Second deleted".to_string(), String::new());

        with_test_state_and_recently_deleted(
            vec![active_note.clone()],
            vec![first_deleted, second_deleted],
            |state| {
                assert_eq!(state.clear_all_recently_deleted_confirmation_count(), None);

                state.request_clear_all_recently_deleted();
                assert_eq!(
                    state.clear_all_recently_deleted_confirmation_count(),
                    Some(2)
                );
                assert_eq!(state.notes_save_revision.get_untracked(), 0);

                state.cancel_clear_all_recently_deleted();
                assert_eq!(state.recently_deleted_notes_untracked().len(), 2);
                assert_eq!(state.notes_save_revision.get_untracked(), 0);

                state.request_clear_all_recently_deleted();
                state.confirm_clear_all_recently_deleted_notes();

                assert_eq!(state.notes_untracked(), vec![active_note]);
                assert!(state.recently_deleted_notes_untracked().is_empty());
                assert_eq!(state.notes_save_revision.get_untracked(), 1);
            },
        );
    }

    #[test]
    fn search_input_and_active_tag_still_drive_note_list_render_model() {
        let mut mobile_note = Note::new(
            "Mobile layout".to_string(),
            "Responsive navigation".to_string(),
        );
        mobile_note.tags = vec!["Mobile".to_string()];
        let desktop_note = Note::new("Desktop writing".to_string(), "Wide workspace".to_string());

        with_test_state(
            vec![mobile_note.clone(), desktop_note],
            ViewportClass::Wide,
            true,
            |state| {
                state.edit_note_search("layout".to_string());
                assert_eq!(state.note_search_input(), "layout");

                state.commit_note_search();
                state.select_active_tag("Mobile".to_string());

                let model = state.note_list_render_model();
                assert_eq!(model.projection.rows.len(), 1);
                assert_eq!(model.projection.rows[0].id, mobile_note.id);
                assert!(model.projection.rows[0].is_selected);
            },
        );
    }

    #[test]
    fn tag_mutations_mark_notes_changed_only_when_metadata_changes() {
        let mut note = Note::new("Tagged".to_string(), String::new());
        note.tags = vec![" Work ".to_string(), "work".to_string()];

        with_test_state(vec![note], ViewportClass::Wide, true, |state| {
            let plan = state.tag_cleanup_plan();
            assert_eq!(state.notes_save_revision.get_untracked(), 0);

            state.apply_tag_cleanup(&plan);
            assert_eq!(state.notes_save_revision.get_untracked(), 1);
            assert_eq!(state.notes_untracked()[0].tags, vec!["Work".to_string()]);

            state.apply_tag_cleanup(&plan);
            assert_eq!(state.notes_save_revision.get_untracked(), 1);

            state.update_selected_tags(Vec::new());
            assert_eq!(state.notes_save_revision.get_untracked(), 2);
            assert!(state.notes_untracked()[0].tags.is_empty());

            state.update_selected_tags(Vec::new());
            assert_eq!(state.notes_save_revision.get_untracked(), 2);
        });
    }

    #[test]
    fn backup_import_replaces_collection_through_app_state_and_marks_notes_changed() {
        let imported_note = Note::new("Imported".to_string(), "Backup content".to_string());
        let backup_json =
            nota_core::backup::export_flat_collection_backup(std::slice::from_ref(&imported_note))
                .unwrap();

        with_test_state(Vec::new(), ViewportClass::Wide, true, |state| {
            state.import_backup_json(&backup_json).unwrap();

            assert_eq!(state.notes_untracked(), vec![imported_note.clone()]);
            assert_eq!(state.selected_id(), Some(imported_note.id));
            assert_eq!(state.notes_save_revision.get_untracked(), 1);
        });
    }

    #[test]
    fn backup_health_summary_makes_missing_and_stale_backups_actionable() {
        with_test_state(Vec::new(), ViewportClass::Wide, true, |state| {
            assert_eq!(state.backup_health_summary(), "No backup yet. Export now");

            let stale_export_at = Utc::now() - chrono::Duration::days(15);
            state.backup_health_record.set(Some(BackupHealthRecord {
                last_successful_export_at: stale_export_at,
            }));
            assert_eq!(
                state.backup_health_summary(),
                format!(
                    "Backup stale {}. Export now",
                    stale_export_at.format("%d/%m/%Y")
                )
            );
        });
    }

    #[test]
    fn invalid_backup_import_does_not_mark_notes_changed() {
        let existing_note = Note::new("Existing".to_string(), String::new());

        with_test_state(
            vec![existing_note.clone()],
            ViewportClass::Wide,
            true,
            |state| {
                assert!(state.import_backup_json("{not valid json").is_err());

                assert_eq!(state.notes_untracked(), vec![existing_note]);
                assert_eq!(state.notes_save_revision.get_untracked(), 0);
            },
        );
    }

    #[test]
    fn global_notifications_replace_and_clear_by_identity() {
        with_test_state(Vec::new(), ViewportClass::Wide, true, |state| {
            state.show_notification("Saving...", NotificationTone::Progress);
            let first = state.notification.get_untracked().unwrap();

            assert_eq!(first.message, "Saving...");
            assert_eq!(first.tone, NotificationTone::Progress);

            state.show_notification("Backup exported", NotificationTone::Success);
            let second = state.notification.get_untracked().unwrap();

            assert_eq!(second.message, "Backup exported");
            assert_eq!(second.tone, NotificationTone::Success);
            assert_ne!(first.id, second.id);

            state.clear_notification(first.id);
            assert!(state.notification.get_untracked().is_some());

            state.clear_notification(second.id);
            assert!(state.notification.get_untracked().is_none());
        });
    }

    #[test]
    fn save_notifications_do_not_replace_domain_notifications() {
        with_test_state(Vec::new(), ViewportClass::Wide, true, |state| {
            state.show_save_notification("Saving...", NotificationTone::Progress);
            assert_eq!(
                state.notification.get_untracked().unwrap().message,
                "Saving..."
            );

            state.show_save_notification("Saved", NotificationTone::Success);
            assert_eq!(state.notification.get_untracked().unwrap().message, "Saved");

            state.show_notification("Backup imported", NotificationTone::Success);
            let backup = state.notification.get_untracked().unwrap();

            state.show_save_notification("Saving...", NotificationTone::Progress);
            assert_eq!(state.notification.get_untracked().unwrap(), backup);

            state.show_save_notification("Saved", NotificationTone::Success);
            assert_eq!(state.notification.get_untracked().unwrap(), backup);
        });
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use crate::storage::SaveStatus;
    use chrono::Utc;
    use leptos::prelude::Owner;
    use nota_core::backup::BackupHealthRecord;
    use nota_core::editor_view::EditorViewMode;
    use nota_core::model::Note;
    use nota_core::responsive_navigation::{StoredNoteListState, ViewportClass};

    #[test]
    fn runtime_startup_exposes_persistence_snapshot_without_raw_signal_access() {
        let active_note = Note::new("Active".to_string(), "Keep writing".to_string());
        let deleted_note = Note::new("Deleted".to_string(), "Recover me".to_string());
        let backup_health_record = BackupHealthRecord {
            last_successful_export_at: Utc::now(),
        };

        Owner::new().with(|| {
            let state = crate::app::AppState::from_startup(AppRuntimeStartup {
                notes: vec![active_note.clone()],
                recently_deleted_notes: vec![deleted_note.clone()],
                is_dark_mode: true,
                viewport_class: ViewportClass::Compact,
                stored_note_list_state: StoredNoteListState::Closed,
                backup_health_record: Some(backup_health_record),
                storage_recovery: None,
            });

            let snapshot = state.persistence_snapshot();

            assert!(snapshot.is_dark_mode);
            assert_eq!(
                snapshot.note_list_state_to_persist,
                Some(StoredNoteListState::Closed)
            );
            assert_eq!(snapshot.notes, vec![active_note]);
            assert_eq!(snapshot.recently_deleted_notes, vec![deleted_note]);
            assert_eq!(snapshot.backup_health_record, Some(backup_health_record));
            assert_eq!(snapshot.save_status, SaveStatus::Saved);
        });
    }

    #[test]
    fn runtime_reclassifies_viewport_and_normalises_view_mode() {
        Owner::new().with(|| {
            let state = crate::app::AppState::from_startup(AppRuntimeStartup {
                notes: Vec::new(),
                recently_deleted_notes: Vec::new(),
                is_dark_mode: false,
                viewport_class: ViewportClass::Wide,
                stored_note_list_state: StoredNoteListState::Closed,
                backup_health_record: None,
                storage_recovery: None,
            });

            state.set_editor_view_mode(EditorViewMode::Split);

            assert!(state.reclassify_viewport(ViewportClass::Compact));

            let snapshot = state.persistence_snapshot();
            assert_eq!(snapshot.viewport_class, ViewportClass::Compact);
            assert_eq!(snapshot.editor_view_mode, EditorViewMode::Write);
            assert_eq!(
                snapshot.note_list_state_to_persist,
                Some(StoredNoteListState::Open)
            );

            assert!(!state.reclassify_viewport(ViewportClass::Compact));
        });
    }
}
