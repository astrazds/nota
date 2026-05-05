mod components;
mod markdown_editing;
mod markdown_preview;
mod model;
mod note_collection;
mod note_discovery;
mod storage;

use components::{ConfirmModal, Editor, Sidebar};

use leptos::prelude::*;
use model::Note;
use storage::{
    flush_pending_save, load_dark_mode, load_notes, load_sidebar_open, save_dark_mode,
    save_sidebar_open, schedule_notes_save, SaveStatus, SaveTimeout,
};
use uuid::Uuid;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

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
    pub fn create_note(self) {
        if let Some(created) = self
            .notes
            .try_update(note_collection::NoteCollection::create_note)
        {
            self.selected_id.set(created.selected_id);
            self.focus_title_request.set(created.should_focus_title);
        }
    }

    pub fn select_note(self, id: Uuid) {
        self.selected_id.set(Some(id));
    }

    pub fn request_delete_note(self, id: Uuid) {
        self.selected_id.set(Some(id));
        self.show_delete_confirm.set(true);
    }

    pub fn confirm_delete_selected_note(self) {
        if let Some(id) = self.selected_id.get_untracked() {
            if let Some(next_selected) = self
                .notes
                .try_update(|notes| note_collection::NoteCollection::delete_note(notes, id))
            {
                self.selected_id.set(next_selected);
            }
        }
        self.show_delete_confirm.set(false);
    }

    pub fn update_selected_title(self, title: String) {
        if let Some(id) = self.selected_id.get_untracked() {
            self.notes.update(|notes| {
                note_collection::NoteCollection::update_title(notes, id, title.clone());
            });
        }
    }

    pub fn update_selected_content(self, content: String) {
        if let Some(id) = self.selected_id.get_untracked() {
            self.notes.update(|notes| {
                note_collection::NoteCollection::update_content(notes, id, content.clone());
            });
        }
    }

    pub fn update_selected_tags(self, tags: Vec<String>) {
        if let Some(id) = self.selected_id.get_untracked() {
            self.notes.update(|notes| {
                note_collection::NoteCollection::update_tags(notes, id, tags.clone());
            });
        }
    }

    pub fn toggle_note_pin(self, id: Uuid) {
        self.notes.update(|notes| {
            note_collection::NoteCollection::toggle_pin(notes, id);
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
    let save_timeout: SaveTimeout = Default::default();
    let save_timeout_for_effect = save_timeout.clone();
    Effect::new(move |_| {
        let notes_to_save = state.notes.get();
        schedule_notes_save(&save_timeout_for_effect, notes_to_save, state.save_status);
    });

    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            let notes_for_visibility = state.notes;
            let status_for_visibility = state.save_status;
            let timeout_for_visibility = save_timeout.clone();
            let visibility_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                flush_pending_save(
                    &timeout_for_visibility,
                    notes_for_visibility,
                    status_for_visibility,
                );
            }) as Box<dyn FnMut(_)>);
            let _ = doc.add_event_listener_with_callback(
                "visibilitychange",
                visibility_listener.as_ref().unchecked_ref(),
            );

            let notes_for_unload = state.notes;
            let status_for_unload = state.save_status;
            let timeout_for_unload = save_timeout.clone();
            let unload_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                flush_pending_save(&timeout_for_unload, notes_for_unload, status_for_unload);
            }) as Box<dyn FnMut(_)>);
            let _ = win.add_event_listener_with_callback(
                "pagehide",
                unload_listener.as_ref().unchecked_ref(),
            );
            let _ = win.add_event_listener_with_callback(
                "beforeunload",
                unload_listener.as_ref().unchecked_ref(),
            );

            visibility_listener.forget();
            unload_listener.forget();
        }
    }

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
