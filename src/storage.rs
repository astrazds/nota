use crate::model::Note;
use crate::sample_notes::debug_starter_notes;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::{GetUntracked, RwSignal, Set, window};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::console;

const STORAGE_KEY: &str = "noter-notes";
const DARK_MODE_KEY: &str = "noter-dark-mode";
const SIDEBAR_OPEN_KEY: &str = "noter-sidebar-open";
pub const NOTES_SAVE_DEBOUNCE_MS: i32 = 300;
pub type SaveTimeout = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Saving,
    Saved,
}

#[derive(Debug, Default, Clone)]
pub struct SaveSession {
    timeout: SaveTimeout,
}

impl SaveSession {
    pub fn schedule_notes_save(&self, notes_to_save: Vec<Note>, status: RwSignal<SaveStatus>) {
        schedule_notes_save(&self.timeout, notes_to_save, status);
    }

    pub fn flush_pending_save(&self, notes: RwSignal<Vec<Note>>, status: RwSignal<SaveStatus>) {
        flush_pending_save(&self.timeout, notes, status);
    }

    pub fn install_page_flush_listeners(
        &self,
        notes: RwSignal<Vec<Note>>,
        status: RwSignal<SaveStatus>,
    ) {
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
        {
            let session_for_visibility = self.clone();
            let visibility_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                session_for_visibility.flush_pending_save(notes, status);
            }) as Box<dyn FnMut(_)>);
            let _ = doc.add_event_listener_with_callback(
                "visibilitychange",
                visibility_listener.as_ref().unchecked_ref(),
            );

            let session_for_unload = self.clone();
            let unload_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                session_for_unload.flush_pending_save(notes, status);
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveLifecycle {
    status: SaveStatus,
    has_pending_save: bool,
}

impl Default for SaveLifecycle {
    fn default() -> Self {
        Self {
            status: SaveStatus::Saved,
            has_pending_save: false,
        }
    }
}

impl SaveLifecycle {
    pub fn note_changed(&mut self) -> SaveStatus {
        self.status = SaveStatus::Saving;
        self.has_pending_save = true;
        self.status
    }

    pub fn save_completed(&mut self) -> SaveStatus {
        self.status = SaveStatus::Saved;
        self.has_pending_save = false;
        self.status
    }

    pub fn flush_pending(&mut self) -> Option<SaveStatus> {
        if self.has_pending_save {
            Some(self.save_completed())
        } else {
            None
        }
    }

    pub fn status(&self) -> SaveStatus {
        self.status
    }
}

fn log_storage_error(operation: &str, key: &str, error: &gloo_storage::errors::StorageError) {
    let message = format!("Storage error ({} {}): {:?}", operation, key, error);
    console::error_1(&message.into());
}

pub fn load_notes() -> Vec<Note> {
    if !notes_storage_key_exists() {
        return debug_starter_notes();
    }

    match LocalStorage::get(STORAGE_KEY) {
        Ok(notes) => notes,
        Err(e) => {
            log_storage_error("load", STORAGE_KEY, &e);
            Vec::new()
        }
    }
}

fn notes_storage_key_exists() -> bool {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
        .is_some()
}

pub fn save_notes(notes: &[Note]) {
    if let Err(e) = LocalStorage::set(STORAGE_KEY, notes) {
        log_storage_error("save", STORAGE_KEY, &e);
    }
}

pub fn flush_pending_save(
    timeout: &SaveTimeout,
    notes: RwSignal<Vec<Note>>,
    status: RwSignal<SaveStatus>,
) {
    let had_pending_save = if let Some((id, _)) = timeout.borrow_mut().take() {
        window().clear_timeout_with_handle(id);
        true
    } else {
        false
    };
    save_notes(&notes.get_untracked());
    let mut lifecycle = SaveLifecycle::default();
    if had_pending_save {
        lifecycle.note_changed();
    }
    status.set(
        lifecycle
            .flush_pending()
            .unwrap_or_else(|| lifecycle.status()),
    );
}

pub fn schedule_notes_save(
    timeout: &SaveTimeout,
    notes_to_save: Vec<Note>,
    status: RwSignal<SaveStatus>,
) {
    let mut lifecycle = SaveLifecycle::default();
    status.set(lifecycle.note_changed());

    if let Some((id, _)) = timeout.borrow_mut().take() {
        window().clear_timeout_with_handle(id);
    }

    let closure = Closure::wrap(Box::new(move || {
        save_notes(&notes_to_save);
        let mut lifecycle = SaveLifecycle::default();
        lifecycle.note_changed();
        status.set(lifecycle.save_completed());
    }) as Box<dyn FnMut()>);

    let id = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            NOTES_SAVE_DEBOUNCE_MS,
        )
        .ok();

    if let Some(id) = id {
        *timeout.borrow_mut() = Some((id, closure));
    }
}

pub fn load_dark_mode() -> bool {
    match LocalStorage::get::<Option<bool>>(DARK_MODE_KEY) {
        Ok(Some(true)) => true,
        Ok(Some(false)) => false,
        Ok(None) => get_system_preference(),
        Err(e) => {
            log_storage_error("load", DARK_MODE_KEY, &e);
            get_system_preference()
        }
    }
}

pub fn save_dark_mode(enabled: bool) {
    if let Err(e) = LocalStorage::set(DARK_MODE_KEY, Some(enabled)) {
        log_storage_error("save", DARK_MODE_KEY, &e);
    }
}

pub fn load_sidebar_open() -> bool {
    match LocalStorage::get(SIDEBAR_OPEN_KEY) {
        Ok(open) => open,
        Err(e) => {
            log_storage_error("load", SIDEBAR_OPEN_KEY, &e);
            true
        }
    }
}

pub fn save_sidebar_open(open: bool) {
    if let Err(e) = LocalStorage::set(SIDEBAR_OPEN_KEY, open) {
        log_storage_error("save", SIDEBAR_OPEN_KEY, &e);
    }
}

fn get_system_preference() -> bool {
    if let Some(media_query_list) = window()
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
    {
        return media_query_list.matches();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_lifecycle_tracks_pending_debounced_save_and_flush() {
        let mut lifecycle = SaveLifecycle::default();

        assert_eq!(lifecycle.status(), SaveStatus::Saved);
        assert_eq!(lifecycle.note_changed(), SaveStatus::Saving);
        assert_eq!(lifecycle.flush_pending(), Some(SaveStatus::Saved));
        assert_eq!(lifecycle.flush_pending(), None);
    }

    #[test]
    fn save_session_owns_a_shared_pending_timeout() {
        let session = SaveSession::default();
        let clone = session.clone();

        assert!(session.timeout.borrow().is_none());
        assert!(clone.timeout.borrow().is_none());
    }
}
