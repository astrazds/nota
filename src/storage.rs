use crate::backup::BackupHealthRecord;
use crate::model::Note;
use crate::sample_notes::debug_starter_notes;
use gloo_storage::errors::StorageError;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::{RwSignal, Set, window};
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::console;

const STORAGE_KEY: &str = "noter-notes";
const RECENTLY_DELETED_STORAGE_KEY: &str = "noter-recently-deleted-notes";
const BACKUP_HEALTH_KEY: &str = "noter-backup-health";
const DARK_MODE_KEY: &str = "noter-dark-mode";
const SIDEBAR_OPEN_KEY: &str = "noter-sidebar-open";
const DOCUMENT_PAGE_FLUSH_EVENTS: &[&str] = &["visibilitychange"];
const WINDOW_PAGE_FLUSH_EVENTS: &[&str] = &["pagehide", "beforeunload"];
const NOTES_SAVE_DEBOUNCE_MS: i32 = 300;
type SaveTimeout = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Saving,
    Saved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupDecision {
    MissingStorage,
    DebugStarterNotes(Vec<Note>),
    SavedEmptyCollection,
    SavedCollection(Vec<Note>),
    CorruptSavedData,
}

impl StartupDecision {
    pub fn into_notes(self) -> Vec<Note> {
        match self {
            Self::MissingStorage | Self::SavedEmptyCollection | Self::CorruptSavedData => {
                Vec::new()
            }
            Self::DebugStarterNotes(notes) | Self::SavedCollection(notes) => notes,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SaveSession {
    timeout: SaveTimeout,
}

impl SaveSession {
    pub fn schedule_notes_save(&self, notes_to_save: Vec<Note>, status: RwSignal<SaveStatus>) {
        schedule_notes_save(&self.timeout, notes_to_save, status);
    }

    pub fn flush_pending_save(&self, notes_to_save: Vec<Note>, status: RwSignal<SaveStatus>) {
        flush_pending_save(&self.timeout, notes_to_save, status);
    }

    pub fn install_page_flush_listeners(
        &self,
        notes: impl Fn() -> Vec<Note> + Clone + 'static,
        status: RwSignal<SaveStatus>,
    ) {
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
        {
            let session_for_visibility = self.clone();
            let notes_for_visibility = notes.clone();
            let visibility_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                session_for_visibility.flush_pending_save(notes_for_visibility(), status);
            }) as Box<dyn FnMut(_)>);
            for event_name in DOCUMENT_PAGE_FLUSH_EVENTS {
                let _ = doc.add_event_listener_with_callback(
                    event_name,
                    visibility_listener.as_ref().unchecked_ref(),
                );
            }

            let session_for_unload = self.clone();
            let notes_for_unload = notes.clone();
            let unload_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                session_for_unload.flush_pending_save(notes_for_unload(), status);
            }) as Box<dyn FnMut(_)>);
            for event_name in WINDOW_PAGE_FLUSH_EVENTS {
                let _ = win.add_event_listener_with_callback(
                    event_name,
                    unload_listener.as_ref().unchecked_ref(),
                );
            }

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

fn log_storage_error(operation: &str, key: &str, error: &StorageError) {
    let message = format!("Storage error ({} {}): {:?}", operation, key, error);
    console::error_1(&message.into());
}

fn log_browser_storage_error(operation: &str, key: &str, error: &wasm_bindgen::JsValue) {
    let message = format!("Storage error ({} {}): {:?}", operation, key, error);
    console::error_1(&message.into());
}

pub fn load_notes() -> Vec<Note> {
    let adapter = BrowserNotesStorage;
    let decision =
        decide_notes_startup(adapter.load_notes_json().as_deref(), debug_starter_notes());

    if matches!(decision, StartupDecision::CorruptSavedData) {
        let message = format!("Storage error (load {STORAGE_KEY}): corrupt saved Notes");
        console::error_1(&message.into());
    }

    decision.into_notes()
}

pub fn load_recently_deleted_notes() -> Vec<Note> {
    let adapter = BrowserNotesStorage;
    decide_recently_deleted_startup(adapter.load_recently_deleted_notes_json().as_deref())
}

pub fn load_backup_health_record() -> Option<BackupHealthRecord> {
    let adapter = BrowserNotesStorage;
    decide_backup_health_startup(adapter.load_backup_health_json().as_deref())
}

fn decide_notes_startup(saved_json: Option<&str>, starter_notes: Vec<Note>) -> StartupDecision {
    let Some(saved_json) = saved_json else {
        return if starter_notes.is_empty() {
            StartupDecision::MissingStorage
        } else {
            StartupDecision::DebugStarterNotes(starter_notes)
        };
    };

    match serde_json::from_str::<Vec<Note>>(saved_json) {
        Ok(notes) if notes.is_empty() => StartupDecision::SavedEmptyCollection,
        Ok(notes) => StartupDecision::SavedCollection(notes),
        Err(_) => StartupDecision::CorruptSavedData,
    }
}

fn decide_recently_deleted_startup(saved_json: Option<&str>) -> Vec<Note> {
    saved_json
        .and_then(|json| serde_json::from_str::<Vec<Note>>(json).ok())
        .unwrap_or_default()
}

fn decide_backup_health_startup(saved_json: Option<&str>) -> Option<BackupHealthRecord> {
    saved_json.and_then(|json| serde_json::from_str::<BackupHealthRecord>(json).ok())
}

struct BrowserNotesStorage;

impl BrowserNotesStorage {
    fn load_notes_json(&self) -> Option<String> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| match storage.get_item(STORAGE_KEY) {
                Ok(value) => value,
                Err(error) => {
                    log_browser_storage_error("load", STORAGE_KEY, &error);
                    None
                }
            })
    }

    fn save_notes(&self, notes: &[Note]) {
        let Ok(notes_json) = serde_json::to_string(notes) else {
            let message = format!("Storage error (save {STORAGE_KEY}): could not serialise Notes");
            console::error_1(&message.into());
            return;
        };

        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
            && let Err(error) = storage.set_item(STORAGE_KEY, &notes_json)
        {
            log_browser_storage_error("save", STORAGE_KEY, &error);
        }
    }

    fn load_recently_deleted_notes_json(&self) -> Option<String> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(
                |storage| match storage.get_item(RECENTLY_DELETED_STORAGE_KEY) {
                    Ok(value) => value,
                    Err(error) => {
                        log_browser_storage_error("load", RECENTLY_DELETED_STORAGE_KEY, &error);
                        None
                    }
                },
            )
    }

    fn save_recently_deleted_notes(&self, notes: &[Note]) {
        let Ok(notes_json) = serde_json::to_string(notes) else {
            let message = format!(
                "Storage error (save {RECENTLY_DELETED_STORAGE_KEY}): could not serialise Notes"
            );
            console::error_1(&message.into());
            return;
        };

        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
            && let Err(error) = storage.set_item(RECENTLY_DELETED_STORAGE_KEY, &notes_json)
        {
            log_browser_storage_error("save", RECENTLY_DELETED_STORAGE_KEY, &error);
        }
    }

    fn load_backup_health_json(&self) -> Option<String> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| match storage.get_item(BACKUP_HEALTH_KEY) {
                Ok(value) => value,
                Err(error) => {
                    log_browser_storage_error("load", BACKUP_HEALTH_KEY, &error);
                    None
                }
            })
    }

    #[cfg(target_arch = "wasm32")]
    fn save_backup_health_record(&self, record: BackupHealthRecord) {
        let Ok(record_json) = serde_json::to_string(&record) else {
            let message = format!(
                "Storage error (save {BACKUP_HEALTH_KEY}): could not serialise Backup health"
            );
            console::error_1(&message.into());
            return;
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = record_json;
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
            && let Err(error) = storage.set_item(BACKUP_HEALTH_KEY, &record_json)
        {
            log_browser_storage_error("save", BACKUP_HEALTH_KEY, &error);
        }
    }
}

pub fn save_notes(notes: &[Note]) {
    BrowserNotesStorage.save_notes(notes);
}

pub fn save_recently_deleted_notes(notes: &[Note]) {
    BrowserNotesStorage.save_recently_deleted_notes(notes);
}

pub fn save_backup_health_record(record: BackupHealthRecord) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = record;

    #[cfg(target_arch = "wasm32")]
    BrowserNotesStorage.save_backup_health_record(record);
}

fn flush_pending_save(
    timeout: &SaveTimeout,
    notes_to_save: Vec<Note>,
    status: RwSignal<SaveStatus>,
) {
    let had_pending_save = if let Some((id, _)) = timeout.borrow_mut().take() {
        window().clear_timeout_with_handle(id);
        true
    } else {
        false
    };
    save_notes(&notes_to_save);
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

fn schedule_notes_save(
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
    match load_optional_preference::<bool>(DARK_MODE_KEY) {
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
    if let Err(e) = LocalStorage::set(DARK_MODE_KEY, enabled) {
        log_storage_error("save", DARK_MODE_KEY, &e);
    }
}

pub fn load_sidebar_open() -> bool {
    match load_optional_preference::<bool>(SIDEBAR_OPEN_KEY) {
        Ok(Some(open)) => open,
        Ok(None) => true,
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

fn load_optional_preference<T>(key: &str) -> Result<Option<T>, StorageError>
where
    T: DeserializeOwned,
{
    optional_preference_value(LocalStorage::get(key))
}

fn optional_preference_value<T>(
    load_result: Result<T, StorageError>,
) -> Result<Option<T>, StorageError> {
    match load_result {
        Ok(value) => Ok(Some(value)),
        Err(StorageError::KeyNotFound(_)) => Ok(None),
        Err(error) => Err(error),
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
    fn startup_uses_saved_empty_collection_instead_of_starter_notes() {
        let starter_note = Note::new("Starter".to_string(), "Only for first run".to_string());

        let decision = decide_notes_startup(Some("[]"), vec![starter_note]);

        assert_eq!(decision, StartupDecision::SavedEmptyCollection);
        assert_eq!(decision.into_notes(), Vec::new());
    }

    #[test]
    fn startup_distinguishes_missing_storage_from_debug_starter_notes() {
        assert_eq!(
            decide_notes_startup(None, Vec::new()),
            StartupDecision::MissingStorage
        );

        let starter_note = Note::new("Starter".to_string(), "Debug first run".to_string());
        assert_eq!(
            decide_notes_startup(None, vec![starter_note.clone()]),
            StartupDecision::DebugStarterNotes(vec![starter_note])
        );
    }

    #[test]
    fn startup_uses_saved_notes_when_storage_contains_a_collection() {
        let saved_note = Note::new("Saved".to_string(), "Existing note".to_string());
        let saved_json = serde_json::to_string(&vec![saved_note.clone()]).unwrap();
        let starter_note = Note::new("Starter".to_string(), "Not used".to_string());

        let decision = decide_notes_startup(Some(&saved_json), vec![starter_note]);

        assert_eq!(decision, StartupDecision::SavedCollection(vec![saved_note]));
    }

    #[test]
    fn startup_treats_corrupt_saved_data_as_distinct_from_first_run() {
        let starter_note = Note::new("Starter".to_string(), "Not used".to_string());

        let decision = decide_notes_startup(Some("{not valid json"), vec![starter_note]);

        assert_eq!(decision, StartupDecision::CorruptSavedData);
        assert_eq!(decision.into_notes(), Vec::new());
    }

    #[test]
    fn recently_deleted_startup_uses_empty_collection_for_missing_or_corrupt_storage() {
        let deleted_note = Note::new("Deleted".to_string(), "Recover me".to_string());
        let saved_json = serde_json::to_string(&vec![deleted_note.clone()]).unwrap();

        assert_eq!(decide_recently_deleted_startup(None), Vec::<Note>::new());
        assert_eq!(
            decide_recently_deleted_startup(Some(&saved_json)),
            vec![deleted_note]
        );
        assert_eq!(
            decide_recently_deleted_startup(Some("{not valid json")),
            Vec::<Note>::new()
        );
    }

    #[test]
    fn backup_health_startup_treats_missing_or_malformed_storage_as_absent() {
        let record = BackupHealthRecord {
            last_successful_export_at: chrono::Utc::now(),
        };
        let saved_json = serde_json::to_string(&record).unwrap();

        assert_eq!(decide_backup_health_startup(None), None);
        assert_eq!(
            decide_backup_health_startup(Some(&saved_json)),
            Some(record)
        );
        assert_eq!(decide_backup_health_startup(Some("{not valid json")), None);
    }

    #[test]
    fn missing_optional_preference_is_not_a_storage_failure() {
        let result = optional_preference_value::<bool>(Err(
            gloo_storage::errors::StorageError::KeyNotFound(DARK_MODE_KEY.to_string()),
        ));

        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn malformed_optional_preference_stays_a_storage_failure() {
        let serde_error = serde_json::from_str::<bool>("not-json").unwrap_err();
        let result = optional_preference_value::<bool>(Err(
            gloo_storage::errors::StorageError::SerdeError(serde_error),
        ));

        assert!(matches!(
            result,
            Err(gloo_storage::errors::StorageError::SerdeError(_))
        ));
    }

    #[test]
    fn save_lifecycle_reports_debounce_completion_as_saved() {
        let mut lifecycle = SaveLifecycle::default();

        assert_eq!(lifecycle.note_changed(), SaveStatus::Saving);
        assert_eq!(lifecycle.save_completed(), SaveStatus::Saved);
        assert_eq!(lifecycle.flush_pending(), None);
    }

    #[test]
    fn page_lifecycle_flush_uses_visibility_and_unload_events() {
        assert_eq!(DOCUMENT_PAGE_FLUSH_EVENTS, &["visibilitychange"]);
        assert_eq!(WINDOW_PAGE_FLUSH_EVENTS, &["pagehide", "beforeunload"]);
    }

    #[test]
    fn save_session_owns_a_shared_pending_timeout() {
        let session = SaveSession::default();
        let clone = session.clone();

        assert!(session.timeout.borrow().is_none());
        assert!(clone.timeout.borrow().is_none());
    }
}
