pub mod backup;
mod components;
mod editor_view;
mod markdown_editing;
mod markdown_preview;
mod model;
mod note_collection;
mod note_discovery;
mod note_list_interaction;
mod note_workspace;
mod responsive_navigation;
mod sample_notes;
mod search_query;
mod storage;
mod tag_rules;
mod theme;

use components::{ConfirmModal, Editor, Sidebar};

use backup::{
    BackupError, BackupHealth, BackupHealthRecord, BackupImportPreview, assess_backup_health,
    backup_file_name, export_flat_collection_backup, preview_flat_collection_backup,
};
use chrono::Utc;
use editor_view::EditorViewMode;
use leptos::prelude::*;
use model::Note;
use note_discovery::{NoteListItem, NoteListProjection, SelectedNoteVisibility};
use note_list_interaction::{
    NoteActionControls, NoteListCommand, NoteListDisplayState, NoteListInteraction,
};
use note_workspace::{FocusIntent, NoteWorkspace, WorkspaceDisplayState};
use responsive_navigation::{
    NoteListPersistence, ResponsiveNavigation, StoredNoteListState, ViewportClass,
    normalize_view_mode,
};
use storage::{
    SaveSession, SaveStatus, load_backup_health_record, load_dark_mode, load_notes,
    load_recently_deleted_notes, load_sidebar_open, save_backup_health_record, save_dark_mode,
    save_recently_deleted_notes, save_sidebar_open,
};
use tag_rules::{TagCleanupPlan, TagSuggestion, collect_note_tags, suggest_existing_tags};
use theme::ThemeSurface;
use uuid::Uuid;
use wasm_bindgen::{JsCast, prelude::Closure};

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

impl AppState {
    pub fn notes(self) -> Vec<Note> {
        self.workspace.get().notes().to_vec()
    }

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

    pub fn note_count(self) -> usize {
        self.workspace.get().notes().len()
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

    pub fn note_list_projection(self) -> NoteListProjection {
        let workspace = self.workspace.get();
        self.note_list_interaction
            .get()
            .project_notes(workspace.notes(), workspace.selected_id())
    }

    pub fn note_list_display_state(self, projection: &NoteListProjection) -> NoteListDisplayState {
        self.note_list_interaction
            .get()
            .display_state(self.workspace.get().notes().len(), projection)
    }

    pub fn selected_note_is_hidden_by_filter(self) -> bool {
        self.note_list_projection().selected_note_visibility
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
        let command = self.note_list_interaction.get_untracked().select_row(id);
        self.apply_note_list_command(command);
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

    pub fn remove_selected_tag(self, tag: &str) {
        let updated = self
            .workspace
            .try_update(|workspace| workspace.remove_selected_tag(tag))
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
        export_flat_collection_backup(self.workspace.get().notes())
    }

    pub fn backup_file_name(self) -> String {
        backup_file_name(Utc::now())
    }

    pub fn backup_health(self) -> BackupHealth {
        assess_backup_health(self.backup_health_record.get(), Utc::now())
    }

    pub fn backup_health_summary(self) -> String {
        match self.backup_health() {
            BackupHealth::Missing => "No backup yet".to_string(),
            BackupHealth::Recent {
                last_successful_export_at,
            } => format!(
                "Last backup {}",
                last_successful_export_at.format("%d/%m/%Y")
            ),
            BackupHealth::Stale {
                last_successful_export_at,
            } => format!(
                "Backup stale since {}",
                last_successful_export_at.format("%d/%m/%Y")
            ),
        }
    }

    pub fn record_backup_exported(self) {
        let record = BackupHealthRecord {
            last_successful_export_at: Utc::now(),
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
        Ok(())
    }

    pub fn preview_backup_import_json(
        self,
        backup_json: &str,
    ) -> Result<BackupImportPreview, BackupError> {
        preview_flat_collection_backup(&self.notes_untracked(), backup_json)
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

fn main() {
    leptos::mount::mount_to_body(|| {
        view! {
            <App />
        }
    });
}

#[component]
fn App() -> impl IntoView {
    let workspace = RwSignal::new(NoteWorkspace::new_with_recently_deleted(
        load_notes(),
        load_recently_deleted_notes(),
    ));
    let notes_save_revision = RwSignal::new(0);
    let is_dark_mode = RwSignal::new(load_dark_mode());
    let viewport_class = RwSignal::new(current_viewport_class());
    let initial_navigation = ResponsiveNavigation::initial(
        viewport_class.get_untracked(),
        StoredNoteListState::from_is_open(load_sidebar_open()),
    );
    let is_sidebar_open = RwSignal::new(initial_navigation.is_note_list_visible());
    let note_list_interaction = RwSignal::new(NoteListInteraction::default());
    let editor_view_mode = RwSignal::new(EditorViewMode::Write);
    let save_status = RwSignal::new(SaveStatus::Saved);
    let backup_health_record = RwSignal::new(load_backup_health_record());
    let notification = RwSignal::new(None);
    let notification_sequence = RwSignal::new(0);

    let state = AppState {
        workspace,
        notes_save_revision,
        is_dark_mode,
        viewport_class,
        is_sidebar_open,
        note_list_interaction,
        editor_view_mode,
        save_status,
        backup_health_record,
        notification,
        notification_sequence,
    };
    provide_context(state);
    install_viewport_listener(state);
    install_quick_capture_shortcut(state);

    // Persist dark mode on change
    Effect::new(move |_| {
        save_dark_mode(is_dark_mode.get());
    });

    // Persist sidebar state on change
    Effect::new(move |_| {
        let navigation = ResponsiveNavigation::current(viewport_class.get(), is_sidebar_open.get());
        if let NoteListPersistence::Persist(stored_state) = navigation.persistence() {
            save_sidebar_open(stored_state.is_open());
        }
    });

    // Persist notes on change
    let save_session = SaveSession::default();
    let save_session_for_effect = save_session.clone();
    Effect::new(move |_| {
        let _ = state.notes_save_revision.get();
        let notes_to_save = state.notes_untracked();
        let recently_deleted_to_save = state.recently_deleted_notes_untracked();
        save_session_for_effect.schedule_notes_save(notes_to_save, state.save_status);
        save_recently_deleted_notes(&recently_deleted_to_save);
    });
    save_session.install_page_flush_listeners(
        move || {
            let notes = state.notes_untracked();
            let recently_deleted = state.recently_deleted_notes_untracked();
            save_recently_deleted_notes(&recently_deleted);
            notes
        },
        state.save_status,
    );

    view! {
        <div
            class=move || {
                format!("{} flex h-screen overflow-hidden", ThemeSurface::RootApp.classes())
            }
            class:dark=move || is_dark_mode.get()
        >
            <Sidebar />
            <Editor />
            <ConfirmModal
                title="Delete Note?"
                message="This can be restored from Recently Deleted."
            />
        </div>
    }
}

fn current_viewport_class() -> ViewportClass {
    web_sys::window()
        .and_then(|win| win.inner_width().ok())
        .and_then(|width| width.as_f64())
        .map(ViewportClass::from_width)
        .unwrap_or(ViewportClass::Wide)
}

fn install_viewport_listener(state: AppState) {
    let Some(win) = web_sys::window() else {
        return;
    };

    let resize_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
        let next_viewport_class = current_viewport_class();
        if state.viewport_class.get_untracked() == next_viewport_class {
            return;
        }

        let mut navigation = state.responsive_navigation_untracked();
        navigation.reclassify_viewport(next_viewport_class);

        state.viewport_class.set(next_viewport_class);
        state.is_sidebar_open.set(navigation.is_note_list_visible());
        state.editor_view_mode.update(|view_mode| {
            *view_mode = normalize_view_mode(next_viewport_class, *view_mode);
        });
    }) as Box<dyn FnMut(_)>);

    let _ =
        win.add_event_listener_with_callback("resize", resize_listener.as_ref().unchecked_ref());
    resize_listener.forget();
}

fn install_quick_capture_shortcut(state: AppState) {
    let Some(win) = web_sys::window() else {
        return;
    };

    let keydown_listener = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
        if !is_quick_capture_shortcut(
            &ev.key(),
            ev.ctrl_key(),
            ev.meta_key(),
            ev.alt_key(),
            ev.shift_key(),
        ) {
            return;
        }

        ev.prevent_default();
        state.quick_capture_note();
    }) as Box<dyn FnMut(_)>);

    let _ =
        win.add_event_listener_with_callback("keydown", keydown_listener.as_ref().unchecked_ref());
    keydown_listener.forget();
}

fn is_quick_capture_shortcut(
    key: &str,
    ctrl_key: bool,
    meta_key: bool,
    alt_key: bool,
    shift_key: bool,
) -> bool {
    key.eq_ignore_ascii_case("n") && (ctrl_key || meta_key) && !alt_key && !shift_key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_test_state<T>(
        notes: Vec<Note>,
        viewport_class: ViewportClass,
        is_sidebar_open: bool,
        test: impl FnOnce(AppState) -> T,
    ) -> T {
        Owner::new().with(|| {
            let state = AppState {
                workspace: RwSignal::new(NoteWorkspace::new(notes)),
                notes_save_revision: RwSignal::new(0),
                is_dark_mode: RwSignal::new(false),
                viewport_class: RwSignal::new(viewport_class),
                is_sidebar_open: RwSignal::new(is_sidebar_open),
                note_list_interaction: RwSignal::new(NoteListInteraction::default()),
                editor_view_mode: RwSignal::new(EditorViewMode::Write),
                save_status: RwSignal::new(SaveStatus::Saved),
                backup_health_record: RwSignal::new(None),
                notification: RwSignal::new(None),
                notification_sequence: RwSignal::new(0),
            };

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
            let row = state.note_list_projection().rows.remove(0);
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
    fn search_input_and_active_tag_still_drive_note_list_projection() {
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

                let projection = state.note_list_projection();
                assert_eq!(projection.rows.len(), 1);
                assert_eq!(projection.rows[0].id, mobile_note.id);
                assert!(projection.rows[0].is_selected);
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

            state.remove_selected_tag("work");
            assert_eq!(state.notes_save_revision.get_untracked(), 2);
            assert!(state.notes_untracked()[0].tags.is_empty());

            state.remove_selected_tag("missing");
            assert_eq!(state.notes_save_revision.get_untracked(), 2);
        });
    }

    #[test]
    fn backup_import_replaces_collection_through_app_state_and_marks_notes_changed() {
        let imported_note = Note::new("Imported".to_string(), "Backup content".to_string());
        let backup_json =
            crate::backup::export_flat_collection_backup(std::slice::from_ref(&imported_note))
                .unwrap();

        with_test_state(Vec::new(), ViewportClass::Wide, true, |state| {
            state.import_backup_json(&backup_json).unwrap();

            assert_eq!(state.notes_untracked(), vec![imported_note.clone()]);
            assert_eq!(state.selected_id(), Some(imported_note.id));
            assert_eq!(state.notes_save_revision.get_untracked(), 1);
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
    fn quick_capture_shortcut_uses_primary_modifier_and_plain_n() {
        assert!(is_quick_capture_shortcut("n", true, false, false, false));
        assert!(is_quick_capture_shortcut("N", false, true, false, false));

        assert!(!is_quick_capture_shortcut("n", false, false, false, false));
        assert!(!is_quick_capture_shortcut("n", true, false, true, false));
        assert!(!is_quick_capture_shortcut("n", true, false, false, true));
        assert!(!is_quick_capture_shortcut("m", true, false, false, false));
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
}
