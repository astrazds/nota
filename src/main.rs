mod components;
mod model;
mod storage;

use components::{ConfirmModal, Editor, Sidebar};

use leptos::prelude::*;
use model::Note;
use std::cell::RefCell;
use std::rc::Rc;
use storage::{
    load_dark_mode, load_notes, load_sidebar_open, save_dark_mode, save_notes, save_sidebar_open,
};
use uuid::Uuid;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

const NOTES_SAVE_DEBOUNCE_MS: i32 = 300;
type TimeoutState = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Saving,
    Saved,
}

fn flush_pending_save(
    timeout: &TimeoutState,
    notes: RwSignal<Vec<Note>>,
    status: RwSignal<SaveStatus>,
) {
    if let Some((id, _)) = timeout.borrow_mut().take() {
        if let Some(win) = window() {
            win.clear_timeout_with_handle(id);
        }
    }
    save_notes(&notes.get_untracked());
    status.set(SaveStatus::Saved);
}

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
    let save_timeout: TimeoutState = Rc::new(RefCell::new(None));
    let save_timeout_for_effect = save_timeout.clone();
    Effect::new(move |_| {
        let notes_to_save = state.notes.get();
        let timeout_ref = save_timeout_for_effect.clone();
        state.save_status.set(SaveStatus::Saving);

        if let Some((id, _)) = timeout_ref.borrow_mut().take() {
            if let Some(win) = window() {
                win.clear_timeout_with_handle(id);
            }
        }

        if let Some(win) = window() {
            let save_status = state.save_status;
            let closure = Closure::wrap(Box::new(move || {
                save_notes(&notes_to_save);
                save_status.set(SaveStatus::Saved);
            }) as Box<dyn FnMut()>);

            let id = win
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    NOTES_SAVE_DEBOUNCE_MS,
                )
                .ok();

            if let Some(id) = id {
                *timeout_ref.borrow_mut() = Some((id, closure));
            }
        }
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
