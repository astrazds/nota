use crate::model::Note;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::window;
use web_sys::console;

const STORAGE_KEY: &str = "noter-notes";
const DARK_MODE_KEY: &str = "noter-dark-mode";
const SIDEBAR_OPEN_KEY: &str = "noter-sidebar-open";

fn log_storage_error(operation: &str, key: &str, error: &gloo_storage::errors::StorageError) {
    let message = format!("Storage error ({} {}): {:?}", operation, key, error);
    console::error_1(&message.into());
}

pub fn load_notes() -> Vec<Note> {
    match LocalStorage::get(STORAGE_KEY) {
        Ok(notes) => notes,
        Err(e) => {
            log_storage_error("load", STORAGE_KEY, &e);
            Vec::new()
        }
    }
}

pub fn save_notes(notes: &[Note]) {
    if let Err(e) = LocalStorage::set(STORAGE_KEY, notes) {
        log_storage_error("save", STORAGE_KEY, &e);
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
