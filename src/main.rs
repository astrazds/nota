mod components;
mod markdown_editing;
mod markdown_preview;
mod model;
mod note_collection;
mod note_discovery;
mod note_workspace;
mod storage;
mod tag_rules;

use components::{ConfirmModal, Editor, Sidebar};

use leptos::prelude::*;
use model::Note;
use note_workspace::NoteWorkspace;
use storage::{
    SaveSession, SaveStatus, load_dark_mode, load_notes, load_sidebar_open, save_dark_mode,
    save_sidebar_open,
};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct AppState {
    pub notes: RwSignal<Vec<Note>>,
    pub selected_id: RwSignal<Option<Uuid>>,
    pub is_dark_mode: RwSignal<bool>,
    pub is_sidebar_open: RwSignal<bool>,
    pub search_query: RwSignal<String>,
    pub active_tag: RwSignal<Option<String>>,
    pub show_delete_confirm: RwSignal<bool>,
    pub show_preview: RwSignal<bool>,
    pub focus_title_request: RwSignal<bool>,
    pub save_status: RwSignal<SaveStatus>,
}

impl AppState {
    pub fn selected_note(self) -> Option<Note> {
        NoteWorkspace::selected_note(&self.notes.get(), self.selected_id.get())
    }

    pub fn create_note(self) {
        if let Some(created) = self.notes.try_update(NoteWorkspace::create_note) {
            self.selected_id.set(created.selected_id);
            self.focus_title_request.set(created.should_focus_title);
        }
    }

    pub fn select_note(self, id: Uuid) {
        self.selected_id.set(Some(id));
    }

    pub fn request_delete_note(self, id: Uuid) {
        let mut selected_id = self.selected_id.get_untracked();
        let mut show_delete_confirm = self.show_delete_confirm.get_untracked();
        NoteWorkspace::request_delete(&mut selected_id, &mut show_delete_confirm, id);
        self.selected_id.set(selected_id);
        self.show_delete_confirm.set(show_delete_confirm);
    }

    pub fn cancel_delete_note(self) {
        let mut show_delete_confirm = self.show_delete_confirm.get_untracked();
        NoteWorkspace::cancel_delete(&mut show_delete_confirm);
        self.show_delete_confirm.set(show_delete_confirm);
    }

    pub fn confirm_delete_selected_note(self) {
        let mut selected_id = self.selected_id.get_untracked();
        let mut show_delete_confirm = self.show_delete_confirm.get_untracked();
        self.notes.update(|notes| {
            NoteWorkspace::confirm_delete(notes, &mut selected_id, &mut show_delete_confirm);
        });
        self.selected_id.set(selected_id);
        self.show_delete_confirm.set(show_delete_confirm);
    }

    pub fn update_selected_title(self, title: String) {
        let selected_id = self.selected_id.get_untracked();
        self.notes.update(|notes| {
            NoteWorkspace::update_selected_title(notes, selected_id, title.clone());
        });
    }

    pub fn update_selected_content(self, content: String) {
        let selected_id = self.selected_id.get_untracked();
        self.notes.update(|notes| {
            NoteWorkspace::update_selected_content(notes, selected_id, content.clone());
        });
    }

    pub fn update_selected_tags(self, tags: Vec<String>) {
        let selected_id = self.selected_id.get_untracked();
        self.notes.update(|notes| {
            NoteWorkspace::update_selected_tags(notes, selected_id, tags.clone());
        });
    }

    pub fn toggle_note_pin(self, id: Uuid) {
        self.notes.update(|notes| {
            NoteWorkspace::toggle_pin(notes, id);
        });
    }

    pub fn toggle_dark_mode(self) {
        self.is_dark_mode.update(|enabled| *enabled = !*enabled);
    }

    pub fn toggle_sidebar(self) {
        self.is_sidebar_open.update(|open| *open = !*open);
    }

    pub fn toggle_preview(self) {
        self.show_preview.update(|show| *show = !*show);
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
    let notes = RwSignal::new(load_notes());
    let selected_id = RwSignal::new(notes.get_untracked().first().map(|n| n.id));
    let is_dark_mode = RwSignal::new(load_dark_mode());
    let is_sidebar_open = RwSignal::new(load_sidebar_open());
    let search_query = RwSignal::new(String::new());
    let active_tag = RwSignal::new(None);
    let show_delete_confirm = RwSignal::new(false);
    let show_preview = RwSignal::new(false);
    let focus_title_request = RwSignal::new(false);
    let save_status = RwSignal::new(SaveStatus::Saved);

    let state = AppState {
        notes,
        selected_id,
        is_dark_mode,
        is_sidebar_open,
        search_query,
        active_tag,
        show_delete_confirm,
        show_preview,
        focus_title_request,
        save_status,
    };
    provide_context(state);

    // Persist dark mode on change
    Effect::new(move |_| {
        save_dark_mode(is_dark_mode.get());
    });

    // Persist sidebar state on change
    Effect::new(move |_| {
        save_sidebar_open(is_sidebar_open.get());
    });

    // Persist notes on change
    let save_session = SaveSession::default();
    let save_session_for_effect = save_session.clone();
    Effect::new(move |_| {
        let notes_to_save = state.notes.get();
        save_session_for_effect.schedule_notes_save(notes_to_save, state.save_status);
    });
    save_session.install_page_flush_listeners(state.notes, state.save_status);

    view! {
        <div
            class="bg-white text-gray-900 dark:bg-apple-dark-bg dark:text-white flex h-screen overflow-hidden transition-colors duration-300"
            class:dark=move || is_dark_mode.get()
        >
            <Sidebar />
            <Editor />
            <ConfirmModal
                show=show_delete_confirm
                title="Delete Note?"
                message="This cannot be undone."
            />
        </div>
    }
}
