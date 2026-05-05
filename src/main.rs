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
mod storage;
mod tag_rules;

use components::{ConfirmModal, Editor, Sidebar};

use editor_view::EditorViewMode;
use leptos::prelude::*;
use model::Note;
use note_discovery::{NoteListItem, NoteListProjection};
use note_list_interaction::{
    NoteActionControls, NoteListCommand, NoteListDisplayState, NoteListInteraction,
};
use note_workspace::{FocusIntent, NoteWorkspace, WorkspaceDisplayState};
use responsive_navigation::{
    NoteListPersistence, ResponsiveNavigation, StoredNoteListState, ViewportClass,
    normalize_view_mode,
};
use storage::{
    SaveSession, SaveStatus, load_dark_mode, load_notes, load_sidebar_open, save_dark_mode,
    save_sidebar_open,
};
use tag_rules::collect_note_tags;
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
}

impl AppState {
    pub fn notes(self) -> Vec<Note> {
        self.workspace.get().notes().to_vec()
    }

    pub fn notes_untracked(self) -> Vec<Note> {
        self.workspace.get_untracked().notes().to_vec()
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

    pub fn note_search_input(self) -> String {
        self.note_list_interaction.get().search_input().to_string()
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

    pub fn create_note(self) {
        self.workspace.update(NoteWorkspace::create_note);
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
    let workspace = RwSignal::new(NoteWorkspace::new(load_notes()));
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

    let state = AppState {
        workspace,
        notes_save_revision,
        is_dark_mode,
        viewport_class,
        is_sidebar_open,
        note_list_interaction,
        editor_view_mode,
        save_status,
    };
    provide_context(state);
    install_viewport_listener(state);

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
        save_session_for_effect.schedule_notes_save(notes_to_save, state.save_status);
    });
    save_session.install_page_flush_listeners(move || state.notes_untracked(), state.save_status);

    view! {
        <div
            class=move || {
                if is_dark_mode.get() {
                    "dark bg-apple-dark-bg text-white flex h-screen overflow-hidden transition-colors duration-300"
                } else {
                    "bg-white text-gray-900 flex h-screen overflow-hidden transition-colors duration-300"
                }
            }
            class:dark=move || is_dark_mode.get()
        >
            <Sidebar />
            <Editor />
            <ConfirmModal
                title="Delete Note?"
                message="This cannot be undone."
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
}
