use crate::backup::{BackupImportPreview, backup_file_name};
use crate::{AppState, NotificationTone, theme, ui_recipes};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{FileReader, HtmlAnchorElement, HtmlInputElement, window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingBackupImport {
    pub backup_json: String,
    pub preview: BackupImportPreview,
}

pub(crate) trait BackupDownloadAdapter {
    fn download(&self, filename: &str, backup_json: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy)]
struct BrowserBackupDownloadAdapter;

impl BackupDownloadAdapter for BrowserBackupDownloadAdapter {
    fn download(&self, filename: &str, backup_json: &str) -> Result<(), String> {
        download_backup(filename, backup_json)
    }
}

pub(crate) fn export_backup_with_adapter(state: AppState, adapter: &impl BackupDownloadAdapter) {
    export_backup_with_adapter_at(state, adapter, Utc::now());
}

pub(crate) fn export_backup_with_adapter_at(
    state: AppState,
    adapter: &impl BackupDownloadAdapter,
    exported_at: DateTime<Utc>,
) {
    export_backup_with_adapter_file_name(
        state,
        adapter,
        &backup_file_name(exported_at),
        exported_at,
    );
}

pub(crate) fn export_backup_with_adapter_file_name(
    state: AppState,
    adapter: &impl BackupDownloadAdapter,
    file_name: &str,
    exported_at: DateTime<Utc>,
) {
    match state.export_backup_json() {
        Ok(backup_json) => match adapter.download(file_name, &backup_json) {
            Ok(()) => {
                state.record_backup_exported_at(exported_at);
                state.show_notification("Backup exported", NotificationTone::Success);
            }
            Err(message) => state.show_notification(message, NotificationTone::Error),
        },
        Err(_) => state.show_notification("Backup export failed", NotificationTone::Error),
    }
}

pub(crate) fn preview_backup_import_read(
    state: AppState,
    pending_backup_import: RwSignal<Option<PendingBackupImport>>,
    backup_json: String,
) {
    match state.preview_backup_import_json(&backup_json) {
        Ok(preview) => {
            pending_backup_import.set(Some(PendingBackupImport {
                backup_json,
                preview,
            }));
            state.show_notification("Backup ready", NotificationTone::Success);
        }
        Err(_) => {
            pending_backup_import.set(None);
            state.show_notification("Backup import failed", NotificationTone::Error);
        }
    }
}

pub(crate) fn confirm_pending_backup_import(
    state: AppState,
    pending_backup_import: RwSignal<Option<PendingBackupImport>>,
) {
    let Some(pending_import) = pending_backup_import.get_untracked() else {
        return;
    };

    match state.import_backup_json(&pending_import.backup_json) {
        Ok(()) => {
            pending_backup_import.set(None);
            state.show_notification("Backup imported", NotificationTone::Success);
        }
        Err(_) => state.show_notification("Backup import failed", NotificationTone::Error),
    }
}

pub(crate) fn cancel_pending_backup_import(
    state: AppState,
    pending_backup_import: RwSignal<Option<PendingBackupImport>>,
) {
    pending_backup_import.set(None);
    state.show_notification("Backup import cancelled", NotificationTone::Progress);
}

#[component]
pub(crate) fn SidebarBackupControls() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let pending_backup_import = RwSignal::new(None::<PendingBackupImport>);

    let export_backup = move |_| {
        export_backup_with_adapter(state, &BrowserBackupDownloadAdapter);
    };

    let import_backup = move |ev| {
        import_backup_from_input_event(state, pending_backup_import, ev);
    };

    let confirm_backup_import = move |_| {
        confirm_pending_backup_import(state, pending_backup_import);
    };

    let cancel_backup_import = move |_| {
        cancel_pending_backup_import(state, pending_backup_import);
    };

    view! {
        <div class="contents">
            <div class=ui_recipes::sidebar_footer>
                <div class="min-w-0 flex-1 leading-4">
                    <div class=ui_recipes::backup_footer_label>"Backup"</div>
                    <div class=ui_recipes::backup_footer_summary>
                        {move || format!("{} notes. {}", state.note_count(), state.backup_health_summary())}
                    </div>
                </div>
                <div class="ml-auto flex flex-none items-center justify-end gap-1.5">
                    <button
                        type="button"
                        class=ui_recipes::backup_footer_button
                        on:click=export_backup
                        aria-label="Export backup"
                    >
                        "Export"
                    </button>
                    <label class=ui_recipes::backup_footer_button aria-label="Import backup">
                        "Import"
                        <input
                            type="file"
                            accept="application/json,.json"
                            class="sr-only"
                            on:change=import_backup
                        />
                    </label>
                </div>
            </div>
            <Show when=move || pending_backup_import.get().is_some()>
                <div class=ui_recipes::backup_import_preview>
                    {move || {
                        pending_backup_import
                            .get()
                            .map(|pending| {
                                view! {
                                    <div class="flex flex-wrap items-center gap-2">
                                        <span class=move || theme::ThemeText::Primary.classes()>
                                            {format!(
                                                "Import {} notes: {} new, {} replace",
                                                pending.preview.total_imported_notes,
                                                pending.preview.notes_to_add,
                                                pending.preview.notes_to_replace
                                            )}
                                        </span>
                                        <button
                                            type="button"
                                            class=ui_recipes::backup_footer_button
                                            on:click=confirm_backup_import
                                        >
                                            "Import"
                                        </button>
                                        <button
                                            type="button"
                                            class=ui_recipes::backup_footer_button
                                            on:click=cancel_backup_import
                                        >
                                            "Cancel"
                                        </button>
                                    </div>
                                }
                            })
                    }}
                </div>
            </Show>
        </div>
    }
}

fn import_backup_from_input_event(
    state: AppState,
    pending_backup_import: RwSignal<Option<PendingBackupImport>>,
    ev: web_sys::Event,
) {
    let input = event_target::<HtmlInputElement>(&ev);
    let Some(file) = input.files().and_then(|files| files.get(0)) else {
        return;
    };

    state.show_notification("Reading backup...", NotificationTone::Progress);
    let Ok(reader) = FileReader::new() else {
        pending_backup_import.set(None);
        state.show_notification("Backup import failed", NotificationTone::Error);
        input.set_value("");
        return;
    };

    let reader_for_load = reader.clone();
    let on_load = Closure::wrap(Box::new(move |_ev: web_sys::ProgressEvent| {
        let Some(backup_json) = reader_for_load
            .result()
            .ok()
            .and_then(|value| value.as_string())
        else {
            pending_backup_import.set(None);
            state.show_notification("Backup import failed", NotificationTone::Error);
            return;
        };

        preview_backup_import_read(state, pending_backup_import, backup_json);
    }) as Box<dyn FnMut(_)>);

    reader.set_onloadend(Some(on_load.as_ref().unchecked_ref()));
    if reader.read_as_text(&file).is_err() {
        pending_backup_import.set(None);
        state.show_notification("Backup import failed", NotificationTone::Error);
    }
    on_load.forget();
    input.set_value("");
}

fn download_backup(filename: &str, backup_json: &str) -> Result<(), String> {
    let document = window()
        .and_then(|window| window.document())
        .ok_or_else(|| "Backup export failed".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Backup export failed".to_string())?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "Backup export failed".to_string())?;

    anchor.set_download(filename);
    anchor.set_href(&format!(
        "data:application/json;charset=utf-8,{}",
        percent_encode_data_url(backup_json)
    ));
    anchor.click();
    Ok(())
}

fn percent_encode_data_url(value: &str) -> String {
    value
        .bytes()
        .fold(String::with_capacity(value.len()), |mut encoded, byte| {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char)
                }
                _ => encoded.push_str(&format!("%{byte:02X}")),
            }
            encoded
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_runtime::AppRuntimeStartup;
    use crate::backup::export_flat_collection_backup;
    use crate::model::Note;
    use crate::responsive_navigation::{StoredNoteListState, ViewportClass};
    use chrono::{TimeZone, Utc};
    use leptos::prelude::{GetUntracked, Owner, RwSignal};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct RecordingDownloadAdapter {
        result: Result<(), String>,
        calls: Rc<RefCell<Vec<(String, String)>>>,
    }

    impl RecordingDownloadAdapter {
        fn succeeds() -> Self {
            Self {
                result: Ok(()),
                calls: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn fails(message: &str) -> Self {
            Self {
                result: Err(message.to_string()),
                calls: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl BackupDownloadAdapter for RecordingDownloadAdapter {
        fn download(&self, filename: &str, backup_json: &str) -> Result<(), String> {
            self.calls
                .borrow_mut()
                .push((filename.to_string(), backup_json.to_string()));
            self.result.clone()
        }
    }

    fn state_with_notes(notes: Vec<Note>) -> crate::AppState {
        crate::AppState::from_startup(AppRuntimeStartup {
            notes,
            recently_deleted_notes: Vec::new(),
            is_dark_mode: false,
            viewport_class: ViewportClass::Wide,
            stored_note_list_state: StoredNoteListState::Open,
            backup_health_record: None,
        })
    }

    #[test]
    fn backup_download_data_url_percent_encodes_json_content() {
        assert_eq!(
            percent_encode_data_url("{\"title\":\"日本語 note\"}"),
            "%7B%22title%22%3A%22%E6%97%A5%E6%9C%AC%E8%AA%9E%20note%22%7D"
        );
    }

    #[test]
    fn backup_export_records_health_only_after_browser_download_succeeds() {
        Owner::new().with(|| {
            let note = Note::new("Export me".to_string(), "Keep this".to_string());
            let state = state_with_notes(vec![note]);
            let adapter = RecordingDownloadAdapter::succeeds();
            let exported_at = Utc.with_ymd_and_hms(2026, 5, 6, 9, 30, 0).unwrap();

            export_backup_with_adapter_file_name(
                state,
                &adapter,
                "noter-backup-2026-05-06.json",
                exported_at,
            );

            let calls = adapter.calls.borrow();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].0, "noter-backup-2026-05-06.json");
            assert!(calls[0].1.contains("\"kind\": \"noter.flat_collection\""));
            assert_eq!(
                state
                    .backup_health_record
                    .get_untracked()
                    .unwrap()
                    .last_successful_export_at,
                exported_at
            );
            let notification = state.notification.get_untracked().unwrap();
            assert_eq!(notification.message, "Backup exported");
            assert_eq!(notification.tone, NotificationTone::Success);
        });
    }

    #[test]
    fn failed_backup_download_does_not_record_backup_health() {
        Owner::new().with(|| {
            let state = state_with_notes(vec![Note::new(
                "Export failure".to_string(),
                "Do not mark healthy".to_string(),
            )]);
            let adapter = RecordingDownloadAdapter::fails("Cannot download backup");

            export_backup_with_adapter(state, &adapter);

            assert!(state.backup_health_record.get_untracked().is_none());
            let notification = state.notification.get_untracked().unwrap();
            assert_eq!(notification.message, "Cannot download backup");
            assert_eq!(notification.tone, NotificationTone::Error);
        });
    }

    #[test]
    fn valid_backup_read_creates_import_preview_and_ready_notification() {
        Owner::new().with(|| {
            let state = state_with_notes(Vec::new());
            let pending = RwSignal::new(None::<PendingBackupImport>);
            let imported = Note::new("Imported".to_string(), "From backup".to_string());
            let backup_json = export_flat_collection_backup(std::slice::from_ref(&imported))
                .expect("backup serializes");

            preview_backup_import_read(state, pending, backup_json.clone());

            let pending_import = pending.get_untracked().unwrap();
            assert_eq!(pending_import.backup_json, backup_json);
            assert_eq!(pending_import.preview.notes_to_add, 1);
            assert_eq!(pending_import.preview.notes_to_replace, 0);
            assert_eq!(pending_import.preview.selected_id, Some(imported.id));
            let notification = state.notification.get_untracked().unwrap();
            assert_eq!(notification.message, "Backup ready");
            assert_eq!(notification.tone, NotificationTone::Success);
        });
    }

    #[test]
    fn invalid_backup_read_leaves_no_pending_import() {
        Owner::new().with(|| {
            let state = state_with_notes(Vec::new());
            let pending = RwSignal::new(None::<PendingBackupImport>);

            preview_backup_import_read(state, pending, "{not valid json".to_string());

            assert!(pending.get_untracked().is_none());
            let notification = state.notification.get_untracked().unwrap();
            assert_eq!(notification.message, "Backup import failed");
            assert_eq!(notification.tone, NotificationTone::Error);
        });
    }

    #[test]
    fn confirming_pending_backup_import_merges_and_clears_preview() {
        Owner::new().with(|| {
            let existing = Note::new("Existing".to_string(), "Will be replaced".to_string());
            let mut replacement = existing.clone();
            replacement.title = "Replacement".to_string();
            replacement.content = "Backup wins".to_string();
            let existing_only = Note::new("Keep".to_string(), "Not in backup".to_string());
            let imported = Note::new("Imported".to_string(), "New note".to_string());
            let backup_json =
                export_flat_collection_backup(&[replacement.clone(), imported.clone()])
                    .expect("backup serializes");
            let state = state_with_notes(vec![existing, existing_only.clone()]);
            let pending = RwSignal::new(None::<PendingBackupImport>);
            preview_backup_import_read(state, pending, backup_json);

            confirm_pending_backup_import(state, pending);

            assert!(pending.get_untracked().is_none());
            assert_eq!(
                state.notes_untracked(),
                vec![replacement, existing_only, imported]
            );
            let notification = state.notification.get_untracked().unwrap();
            assert_eq!(notification.message, "Backup imported");
            assert_eq!(notification.tone, NotificationTone::Success);
        });
    }

    #[test]
    fn cancelling_pending_backup_import_clears_preview_without_importing() {
        Owner::new().with(|| {
            let state = state_with_notes(Vec::new());
            let pending = RwSignal::new(None::<PendingBackupImport>);
            let imported = Note::new("Imported".to_string(), "Should not import".to_string());
            let backup_json = export_flat_collection_backup(std::slice::from_ref(&imported))
                .expect("backup serializes");
            preview_backup_import_read(state, pending, backup_json);

            cancel_pending_backup_import(state, pending);

            assert!(pending.get_untracked().is_none());
            assert!(state.notes_untracked().is_empty());
            let notification = state.notification.get_untracked().unwrap();
            assert_eq!(notification.message, "Backup import cancelled");
            assert_eq!(notification.tone, NotificationTone::Progress);
        });
    }
}
