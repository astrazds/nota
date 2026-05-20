use crate::backup::BackupHealthRecord;
use crate::model::Note;
use crate::sample_notes::debug_starter_notes;
#[cfg(target_arch = "wasm32")]
use crate::storage_recovery::plan_collection_save_from_json;
use crate::storage_recovery::{CollectionStartup, StorageRecoveryState, StoredCollectionPayload};
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
const PREVIOUS_STORAGE_KEY: &str = "noter-notes-previous";
const PREVIOUS_RECENTLY_DELETED_STORAGE_KEY: &str = "noter-recently-deleted-notes-previous";
#[cfg(target_arch = "wasm32")]
const CORRUPT_STORAGE_KEY: &str = "noter-notes-corrupt-last";
#[cfg(target_arch = "wasm32")]
const CORRUPT_RECENTLY_DELETED_STORAGE_KEY: &str = "noter-recently-deleted-notes-corrupt-last";
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

#[derive(Debug, Default, Clone)]
pub struct SaveSession {
    timeout: SaveTimeout,
}

impl SaveSession {
    pub fn schedule_collection_save(
        &self,
        notes_to_save: Vec<Note>,
        recently_deleted_notes_to_save: Vec<Note>,
        status: RwSignal<SaveStatus>,
    ) {
        schedule_collection_save(
            &self.timeout,
            notes_to_save,
            recently_deleted_notes_to_save,
            status,
        );
    }

    pub fn flush_pending_collection_save(
        &self,
        notes_to_save: Vec<Note>,
        recently_deleted_notes_to_save: Vec<Note>,
        status: RwSignal<SaveStatus>,
    ) {
        flush_pending_collection_save(
            &self.timeout,
            notes_to_save,
            recently_deleted_notes_to_save,
            status,
        );
    }

    pub fn install_page_flush_listeners(
        &self,
        collection: impl Fn() -> (Vec<Note>, Vec<Note>) + Clone + 'static,
        status: RwSignal<SaveStatus>,
    ) {
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
        {
            let session_for_visibility = self.clone();
            let collection_for_visibility = collection.clone();
            let visibility_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                let (notes, recently_deleted_notes) = collection_for_visibility();
                session_for_visibility.flush_pending_collection_save(
                    notes,
                    recently_deleted_notes,
                    status,
                );
            }) as Box<dyn FnMut(_)>);
            for event_name in DOCUMENT_PAGE_FLUSH_EVENTS {
                let _ = doc.add_event_listener_with_callback(
                    event_name,
                    visibility_listener.as_ref().unchecked_ref(),
                );
            }

            let session_for_unload = self.clone();
            let collection_for_unload = collection.clone();
            let unload_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                let (notes, recently_deleted_notes) = collection_for_unload();
                session_for_unload.flush_pending_collection_save(
                    notes,
                    recently_deleted_notes,
                    status,
                );
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

pub fn load_collection_startup() -> CollectionStartup {
    let adapter = BrowserNotesStorage;
    let startup = crate::storage_recovery::decide_collection_startup(
        StoredCollectionPayload {
            notes_json: adapter.load_notes_json(),
            recently_deleted_notes_json: adapter.load_recently_deleted_notes_json(),
        },
        StoredCollectionPayload {
            notes_json: adapter.load_previous_notes_json(),
            recently_deleted_notes_json: adapter.load_previous_recently_deleted_notes_json(),
        },
        debug_starter_notes(),
    );

    if matches!(startup, CollectionStartup::Recovery(_)) {
        let message = format!("Storage error (load {STORAGE_KEY}): corrupt saved Notes");
        console::error_1(&message.into());
    }

    startup
}

pub fn load_backup_health_record() -> Option<BackupHealthRecord> {
    let adapter = BrowserNotesStorage;
    decide_backup_health_startup(adapter.load_backup_health_json().as_deref())
}

fn decide_backup_health_startup(saved_json: Option<&str>) -> Option<BackupHealthRecord> {
    saved_json.and_then(|json| serde_json::from_str::<BackupHealthRecord>(json).ok())
}

struct BrowserNotesStorage;

impl BrowserNotesStorage {
    fn load_notes_json(&self) -> Option<String> {
        self.load_json(STORAGE_KEY)
    }

    fn load_recently_deleted_notes_json(&self) -> Option<String> {
        self.load_json(RECENTLY_DELETED_STORAGE_KEY)
    }

    fn load_previous_notes_json(&self) -> Option<String> {
        self.load_json(PREVIOUS_STORAGE_KEY)
    }

    fn load_previous_recently_deleted_notes_json(&self) -> Option<String> {
        self.load_json(PREVIOUS_RECENTLY_DELETED_STORAGE_KEY)
    }

    #[cfg(target_arch = "wasm32")]
    fn load_corrupt_notes_json(&self) -> Option<String> {
        self.load_json(CORRUPT_STORAGE_KEY)
    }

    #[cfg(target_arch = "wasm32")]
    fn load_corrupt_recently_deleted_notes_json(&self) -> Option<String> {
        self.load_json(CORRUPT_RECENTLY_DELETED_STORAGE_KEY)
    }

    fn load_json(&self, key: &str) -> Option<String> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| match storage.get_item(key) {
                Ok(value) => value,
                Err(error) => {
                    log_browser_storage_error("load", key, &error);
                    None
                }
            })
    }

    #[cfg(target_arch = "wasm32")]
    fn save_note_collection(&self, notes: &[Note], recently_deleted_notes: &[Note]) {
        let Ok(notes_json) = serde_json::to_string(notes) else {
            let message = format!("Storage error (save {STORAGE_KEY}): could not serialise Notes");
            console::error_1(&message.into());
            return;
        };
        let Ok(recently_deleted_notes_json) = serde_json::to_string(recently_deleted_notes) else {
            let message = format!(
                "Storage error (save {RECENTLY_DELETED_STORAGE_KEY}): could not serialise Notes"
            );
            console::error_1(&message.into());
            return;
        };

        let current = StoredCollectionPayload {
            notes_json: self.load_notes_json(),
            recently_deleted_notes_json: self.load_recently_deleted_notes_json(),
        };
        let Ok(plan) =
            plan_collection_save_from_json(current, notes_json, recently_deleted_notes_json)
        else {
            let message = format!("Storage error (save {STORAGE_KEY}): invalid next Notes payload");
            console::error_1(&message.into());
            return;
        };

        if let Some(previous_notes_json) = plan.previous_notes_json {
            self.save_raw(PREVIOUS_STORAGE_KEY, &previous_notes_json);
        }
        if let Some(previous_recently_deleted_notes_json) =
            plan.previous_recently_deleted_notes_json
        {
            self.save_raw(
                PREVIOUS_RECENTLY_DELETED_STORAGE_KEY,
                &previous_recently_deleted_notes_json,
            );
        }
        self.save_raw(STORAGE_KEY, &plan.next_notes_json);
        self.save_raw(
            RECENTLY_DELETED_STORAGE_KEY,
            &plan.next_recently_deleted_notes_json,
        );
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
    fn save_raw(&self, key: &str, value: &str) {
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
            && let Err(error) = storage.set_item(key, value)
        {
            log_browser_storage_error("save", key, &error);
        }
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

pub fn save_note_collection(notes: &[Note], recently_deleted_notes: &[Note]) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = notes;
        let _ = recently_deleted_notes;
    }

    #[cfg(target_arch = "wasm32")]
    BrowserNotesStorage.save_note_collection(notes, recently_deleted_notes);
}

pub fn quarantine_corrupt_payloads(recovery: &StorageRecoveryState) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = recovery;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let adapter = BrowserNotesStorage;
        if let Some(corrupt_notes_json) = &recovery.corrupt_notes_json {
            adapter.save_raw(CORRUPT_STORAGE_KEY, corrupt_notes_json);
        }
        if let Some(corrupt_recently_deleted_notes_json) =
            &recovery.corrupt_recently_deleted_notes_json
        {
            adapter.save_raw(
                CORRUPT_RECENTLY_DELETED_STORAGE_KEY,
                corrupt_recently_deleted_notes_json,
            );
        }
    }
}

pub fn has_quarantined_corrupt_payloads() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }

    #[cfg(target_arch = "wasm32")]
    {
        let adapter = BrowserNotesStorage;
        adapter.load_corrupt_notes_json().is_some()
            || adapter.load_corrupt_recently_deleted_notes_json().is_some()
    }
}

pub fn save_backup_health_record(record: BackupHealthRecord) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = record;

    #[cfg(target_arch = "wasm32")]
    BrowserNotesStorage.save_backup_health_record(record);
}

fn flush_pending_collection_save(
    timeout: &SaveTimeout,
    notes_to_save: Vec<Note>,
    recently_deleted_notes_to_save: Vec<Note>,
    status: RwSignal<SaveStatus>,
) {
    let had_pending_save = if let Some((id, _)) = timeout.borrow_mut().take() {
        window().clear_timeout_with_handle(id);
        true
    } else {
        false
    };
    save_note_collection(&notes_to_save, &recently_deleted_notes_to_save);
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

fn schedule_collection_save(
    timeout: &SaveTimeout,
    notes_to_save: Vec<Note>,
    recently_deleted_notes_to_save: Vec<Note>,
    status: RwSignal<SaveStatus>,
) {
    let mut lifecycle = SaveLifecycle::default();
    status.set(lifecycle.note_changed());

    if let Some((id, _)) = timeout.borrow_mut().take() {
        window().clear_timeout_with_handle(id);
    }

    let closure = Closure::wrap(Box::new(move || {
        save_note_collection(&notes_to_save, &recently_deleted_notes_to_save);
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
