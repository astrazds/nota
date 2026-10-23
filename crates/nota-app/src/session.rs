use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use chrono::Utc;
use nota_core::backup::{BackupHealthRecord, backup_file_name};
use nota_core::editor_view::EditorViewMode;
use nota_core::exchange::{ExchangeError, ImportDestination, export_notebook, stage_import};
use nota_core::markdown_editing::{ByteSelection, apply_markdown_command};
use nota_core::tag_rules::collect_note_tags;
use nota_core::transition::ThemePreference;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use nota_core::note_workspace::FocusIntent;
use nota_core::product::{
    Prompt, clear_deleted_prompt, delete_prompt, permanent_delete_prompt, tag_cleanup_prompt,
};

use crate::app::{AppModel, AppliedImport, NotificationTone, SaveStatus};
use crate::command::{Command, Effect, FormatEdit, TextSelection, decode_command};
use crate::frame::{DeletedRow, Frame, FrameNotification, FrameRecovery, FrameRow};
use crate::persistence::PersistenceWorker;
use crate::preview::{PreviewLayout, external_navigation_target, preview_document};
use crate::storage::{
    CollectionEnvelope, LoadOutcome, NativeRecovery, NativeStore, Preferences, StorageError,
    write_atomic,
};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenOptionsJson {
    pub data_directory: Option<PathBuf>,
}

#[derive(Debug)]
pub struct SessionError {
    pub code: &'static str,
    pub message: String,
}

impl SessionError {
    pub fn new(code: &'static str, message: impl ToString) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    pub fn reply(&self) -> Value {
        json!({"ok": false, "error": {"code": self.code, "message": self.message}})
    }
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SessionError {}
impl From<StorageError> for SessionError {
    fn from(error: StorageError) -> Self {
        Self::new("storage", error)
    }
}

/// Owns one writable profile. Callers serialize commands and flush before closing.
pub struct Session {
    pub app: AppModel,
    pub store: NativeStore,
    pub recovery: Option<NativeRecovery>,
    preferences: Preferences,
    worker: PersistenceWorker,
    completions: Receiver<Result<u64, StorageError>>,
    edit_sequence: u64,
    metadata_pending: bool,
    confirm: Option<ConfirmKind>,
    _profile_lock: File,
}

impl Session {
    pub fn open(options: OpenOptionsJson) -> Result<Self, SessionError> {
        Self::open_with_completion(options, || {})
    }

    pub fn open_with_completion(
        options: OpenOptionsJson,
        completed: impl Fn() + Send + 'static,
    ) -> Result<Self, SessionError> {
        let store = match options.data_directory {
            Some(path) if path.is_absolute() => NativeStore::at(path),
            Some(_) => {
                return Err(SessionError::new(
                    "invalid_path",
                    "Data directory must be absolute",
                ));
            }
            None => NativeStore::discover()?,
        };
        std::fs::create_dir_all(store.data_dir()).map_err(|e| SessionError::new("storage", e))?;
        let profile_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(store.data_dir().join("profile.lock"))
            .map_err(|e| SessionError::new("storage", e))?;
        profile_lock
            .try_lock()
            .map_err(|e| SessionError::new("profile_in_use", e))?;
        let (app, recovery, preferences) = load_profile(&store)?;
        let (sender, completions) = mpsc::channel();
        let worker = PersistenceWorker::start(store.clone(), move |result| {
            if sender.send(result).is_ok() {
                completed();
            }
        });
        Ok(Self {
            app,
            store,
            recovery,
            preferences,
            worker,
            completions,
            edit_sequence: 0,
            metadata_pending: false,
            confirm: None,
            _profile_lock: profile_lock,
        })
    }

    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    fn schedule_save(&mut self) -> Result<(), SessionError> {
        if self
            .worker
            .schedule(self.app.revision(), self.app.collection())
        {
            Ok(())
        } else {
            self.app
                .note_save_failed("Persistence worker stopped".into());
            Err(SessionError::new("storage", "Persistence worker stopped"))
        }
    }
    pub fn poll(&mut self) {
        while let Ok(result) = self.completions.try_recv() {
            match result {
                Ok(revision) => {
                    if self.metadata_pending
                        && let Err(error) = self.persist_metadata()
                    {
                        self.app.note_save_failed(error.to_string());
                    } else {
                        self.app.note_saved(revision);
                    }
                }
                Err(error) => {
                    self.app.note_save_failed(error.to_string());
                }
            }
        }
    }

    pub fn flush(&mut self) -> Result<(), SessionError> {
        self.poll();
        if self.recovery.is_some() {
            return Ok(());
        }
        let result = self.worker.flush().and_then(|revision| {
            self.persist_metadata()?;
            Ok(revision)
        });
        self.poll();
        match result {
            Ok(revision) => {
                self.app.note_saved(revision);
                Ok(())
            }
            Err(error) => {
                self.app.note_save_failed(error.to_string());
                Err(error.into())
            }
        }
    }

    fn persist_metadata(&mut self) -> Result<(), StorageError> {
        self.preferences.theme = self.app.theme;
        self.store.save_preferences(&self.preferences)?;
        self.store
            .persist_backup_health(self.app.backup_health.as_ref())?;
        self.metadata_pending = false;
        Ok(())
    }

    /// Applies one session command.
    ///
    /// A second command enum is not a session command.
    ///
    /// ```compile_fail
    /// enum Other {
    ///     Resize,
    /// }
    /// fn reject(session: &mut nota_app::session::Session) {
    ///     let _ = session.apply(Other::Resize);
    /// }
    /// ```
    pub fn apply(&mut self, command: Command) -> Result<Effect, SessionError> {
        self.poll();
        if self.recovery.is_some() && !allowed_during_recovery(&command) {
            return Err(SessionError::new(
                "storage_recovery",
                "Resolve Storage Recovery before editing",
            ));
        }
        let revision = self.app.revision();
        let effect = self.dispatch(command)?;
        if self.app.revision() != revision {
            self.schedule_save()?;
        }
        Ok(effect)
    }

    pub fn execute_json(&mut self, value: Value) -> Result<Value, SessionError> {
        let command = decode_command(&value)
            .map_err(|error| SessionError::new("invalid_argument", error.message))?;
        let effect = self.apply(command)?;
        Ok(self.reply_json(effect))
    }

    pub fn take_focus_title(&mut self) -> bool {
        self.app.workspace.take_focus_intent() == FocusIntent::NoteTitle
    }

    pub fn tag_suggestion_labels(&self, input: &str) -> Vec<String> {
        self.app
            .tag_suggestions(input)
            .into_iter()
            .map(|suggestion| suggestion.completed_input)
            .collect()
    }

    fn dispatch(&mut self, command: Command) -> Result<Effect, SessionError> {
        match command {
            Command::Snapshot {} => {}
            Command::NewNote {} => {
                self.app.quick_capture();
            }
            Command::Flush {} => self.flush()?,
            Command::Resize { width_px } => {
                self.app.resize(width_px);
            }
            Command::SetViewMode { mode } => {
                self.app.set_view_mode(mode);
            }
            Command::SetTheme { theme } => self.store_theme(theme)?,
            Command::ToggleTheme {} => {
                let theme = self.app.toggle_theme();
                self.store_theme(theme)?;
            }
            Command::ToggleNavigation {} => {
                self.app.toggle_navigation();
            }
            Command::SelectNote { id } => {
                self.require_note(id, false)?;
                self.app.select_note(id);
            }
            Command::EditNote {
                id,
                edit_sequence,
                title,
                content,
                tags_input,
            } => {
                self.require_note(id, false)?;
                if edit_sequence <= self.edit_sequence {
                    return Err(SessionError::new(
                        "stale_edit",
                        "Edit sequence has already been acknowledged",
                    ));
                }
                let selected = self.app.workspace.selected_id();
                self.app.select_note(id);
                self.app.update_title(title);
                self.app.update_content(content);
                self.app.update_tags(tags_input);
                if let Some(selected) = selected {
                    self.app.select_note(selected);
                }
                self.edit_sequence = edit_sequence;
            }
            Command::Search { query } => {
                self.app.edit_search(query);
                self.app.commit_search();
            }
            Command::FilterTag { tag } => match tag {
                Some(tag) => {
                    self.app.select_tag(tag);
                }
                None => {
                    self.app.clear_tag();
                }
            },
            Command::FilterPinned { pinned } => {
                self.app.set_pinned_filter(pinned);
            }
            Command::TogglePin { id } => {
                self.require_note(id, false)?;
                self.app.toggle_pin(id);
            }
            Command::AskDelete { id } => {
                self.require_note(id, false)?;
                self.clear_prompt();
                self.app.request_delete(id);
                self.confirm = Some(ConfirmKind::Delete(id));
            }
            Command::AskPermanentDelete { id } => {
                self.require_note(id, true)?;
                self.clear_prompt();
                self.confirm = Some(ConfirmKind::Permanent(id));
            }
            Command::AskClearDeleted {} => {
                self.clear_prompt();
                if self.app.request_clear_all() {
                    self.confirm = Some(ConfirmKind::Clear);
                }
            }
            Command::AcceptConfirm {} => match self.confirm.take() {
                Some(ConfirmKind::Delete(id)) => {
                    self.require_note(id, false)?;
                    self.app.confirm_delete();
                }
                Some(ConfirmKind::Permanent(id)) => {
                    self.app.permanently_delete(id);
                }
                Some(ConfirmKind::Clear) => {
                    self.app.confirm_clear_all();
                }
                Some(ConfirmKind::Import) => self.commit_import()?,
                None => {
                    return Err(SessionError::new(
                        "no_pending_import",
                        "Choose a backup first",
                    ));
                }
            },
            Command::DismissConfirm {} => match self.confirm.take() {
                Some(ConfirmKind::Delete(_)) => {
                    self.app.cancel_delete();
                }
                Some(ConfirmKind::Clear) => {
                    self.app.cancel_clear_all();
                }
                Some(ConfirmKind::Import) => {
                    self.app.cancel_import();
                }
                Some(ConfirmKind::Permanent(_)) | None => {}
            },
            Command::RestoreNote { id } => {
                self.require_note(id, true)?;
                self.app.restore_recently_deleted(id);
            }
            Command::ImportNotebook { json } => {
                self.clear_prompt();
                let destination = ImportDestination {
                    notes: self.app.workspace.notes(),
                    recently_deleted: self.app.workspace.recently_deleted_notes(),
                    storage_recovery: self.recovery.is_some(),
                };
                match stage_import(&json, destination) {
                    Ok(staged) => {
                        self.app.stage_import(staged);
                        self.confirm = Some(ConfirmKind::Import);
                    }
                    Err(error) => return Err(exchange_error(error)),
                }
            }
            Command::ExportNotebook { path } => {
                let exported = self.notebook_export()?;
                write_atomic(&path, exported.json.as_bytes())?;
                self.record_successful_notes_export(exported.health)?;
            }
            Command::RestorePrevious {} => {
                resolve_recovery(&mut self.app, &self.store, &mut self.recovery, false)?;
            }
            Command::StartEmpty {} => {
                resolve_recovery(&mut self.app, &self.store, &mut self.recovery, true)?;
            }
            Command::Format {
                content,
                selection,
                kind,
            } => {
                let selection = selection_bytes(&content, selection)?;
                let formatted = apply_markdown_command(&content, selection, kind);
                let caret_utf16 = formatted.content[..formatted.caret_byte]
                    .encode_utf16()
                    .count();
                return Ok(Effect::Edit(FormatEdit {
                    content: formatted.content,
                    caret_utf16,
                    caret_byte: formatted.caret_byte,
                }));
            }
            Command::Preview { title, content } => {
                let dark = self.app.theme == ThemePreference::Dark;
                let layout = if self.app.view_mode != EditorViewMode::Split {
                    PreviewLayout::Reading
                } else if self.app.layout.title_px < 39.0 {
                    PreviewLayout::SplitNarrow
                } else {
                    PreviewLayout::Split
                };
                return Ok(Effect::Preview(preview_document(
                    &title, &content, dark, layout,
                )));
            }
            Command::ExternalNavigation {
                uri,
                user_activated,
            } => {
                return Ok(Effect::External(
                    external_navigation_target(&uri, user_activated).map(|uri| uri.to_string()),
                ));
            }
            Command::ApplyTagCleanup {} => {
                self.app.apply_tag_cleanup();
            }
            Command::AcceptTagSuggestion { input } => {
                self.app.accept_tag_suggestion(input);
            }
            Command::StartEditTags {} => {
                self.app.start_edit_tags();
            }
            Command::FinishEditTags {} => {
                self.app.finish_edit_tags();
            }
            Command::DismissNotification { generation } => {
                self.app.dismiss_notification(generation);
            }
        }
        Ok(Effect::Frame(self.frame()))
    }

    fn store_theme(&mut self, theme: ThemePreference) -> Result<(), SessionError> {
        let preferences = Preferences {
            theme,
            ..self.preferences.clone()
        };
        self.store.save_preferences(&preferences)?;
        self.preferences = preferences;
        self.app.theme = theme;
        Ok(())
    }

    fn clear_prompt(&mut self) {
        self.confirm = None;
        self.app.cancel_delete();
        self.app.cancel_clear_all();
        self.app.clear_pending_import();
    }

    fn commit_import(&mut self) -> Result<(), SessionError> {
        if self.app.staged_import().is_none() {
            return Err(SessionError::new(
                "no_pending_import",
                "Choose a backup first",
            ));
        }
        if let Some(recovery) = &self.recovery {
            self.store.preserve_corrupt(recovery, Utc::now())?;
        }
        match self.app.commit_staged_import().map_err(exchange_error)? {
            Some(AppliedImport::Merged) => {
                self.recovery = None;
            }
            Some(AppliedImport::Replaced) => {
                self.metadata_pending = true;
                self.recovery = None;
            }
            None => {
                return Err(SessionError::new(
                    "no_pending_import",
                    "Choose a backup first",
                ));
            }
        }
        Ok(())
    }

    pub fn notebook_export(&self) -> Result<nota_core::exchange::NotebookExport, SessionError> {
        export_notebook(
            self.app.workspace.notes(),
            self.app.workspace.recently_deleted_notes(),
            self.app.theme,
            Utc::now(),
        )
        .map_err(|error| SessionError::new("backup", error))
    }

    pub fn record_successful_notes_export(
        &mut self,
        health: BackupHealthRecord,
    ) -> Result<(), SessionError> {
        self.store.save_backup_health(&health)?;
        self.app.backup_exported(health.last_successful_export_at);
        Ok(())
    }

    fn require_note(&self, id: Uuid, deleted: bool) -> Result<(), SessionError> {
        let notes = if deleted {
            self.app.workspace.recently_deleted_notes()
        } else {
            self.app.workspace.notes()
        };
        if notes.iter().any(|note| note.id == id) {
            Ok(())
        } else {
            Err(SessionError::new(
                "note_not_found",
                "The requested note no longer exists",
            ))
        }
    }

    pub fn reply_json(&self, effect: Effect) -> Value {
        let result = match &effect {
            Effect::Frame(_) => Value::Null,
            Effect::Edit(edit) => json!({
                "content": edit.content,
                "caret_utf16": edit.caret_utf16,
                "caret_byte": edit.caret_byte
            }),
            Effect::Preview(html) => json!({ "html": html }),
            Effect::External(uri) => json!({ "uri": uri }),
        };
        json!({"ok": true, "frame": self.frame(), "result": result})
    }

    pub fn frame(&self) -> Frame {
        let rendered = self.app.note_list_render_model();
        let list = self
            .app
            .note_list
            .list_copy(self.app.workspace.notes(), self.app.workspace.selected_id());
        let rows = rendered
            .projection
            .rows
            .into_iter()
            .map(|row| FrameRow {
                id: row.id,
                title: row.display_title,
                preview: row.preview,
                date: row.display_date,
                is_pinned: row.is_pinned,
            })
            .collect();
        let recently_deleted = self
            .app
            .workspace
            .recently_deleted_notes()
            .iter()
            .map(|note| DeletedRow {
                id: note.id,
                title: note.display_title().to_string(),
            })
            .collect();
        let notification = self
            .app
            .notification
            .as_ref()
            .map(|notification| FrameNotification {
                message: notification.message.clone(),
                tone: match notification.tone {
                    NotificationTone::Progress => "progress",
                    NotificationTone::Success => "success",
                    NotificationTone::Error => "error",
                }
                .to_string(),
            });
        let health = self.app.backup_health_status(Utc::now()).sentences();
        let confirm = match self.confirm {
            Some(ConfirmKind::Delete(_)) => {
                let title = self
                    .app
                    .workspace
                    .delete_confirmation_title()
                    .unwrap_or("New Note");
                Some(delete_prompt(title))
            }
            Some(ConfirmKind::Permanent(_)) => Some(permanent_delete_prompt()),
            Some(ConfirmKind::Clear) => {
                let count = self
                    .app
                    .workspace
                    .clear_all_recently_deleted_confirmation_count()
                    .unwrap_or(0);
                Some(clear_deleted_prompt(count))
            }
            Some(ConfirmKind::Import) => self.app.pending_import().map(|pending| Prompt {
                title: pending.title,
                body: pending.body,
                action: pending.action,
            }),
            None => None,
        };
        let plan = self.app.workspace.tag_cleanup_plan();
        let tag_cleanup = if plan.is_empty() {
            None
        } else {
            let detail = plan
                .changes
                .iter()
                .map(|change| {
                    format!(
                        "{} -> {}",
                        change.before.join(", "),
                        change.after.join(", ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Some(tag_cleanup_prompt(detail))
        };
        Frame {
            revision: self.app.revision(),
            edit_sequence: self.edit_sequence,
            selected_note: self.app.workspace.selected_note(),
            rows,
            recently_deleted,
            search_input: self.app.note_list.search_input().to_string(),
            active_tag: self.app.note_list.active_tag().map(str::to_string),
            tags: collect_note_tags(self.app.workspace.notes()),
            pinned_only: self.app.note_list.pinned_only(),
            view_mode: match self.app.view_mode {
                EditorViewMode::Write => "write",
                EditorViewMode::Preview => "preview",
                EditorViewMode::Split => "split",
            }
            .to_string(),
            theme: self.app.theme,
            save_status: match self.app.save_status {
                SaveStatus::Saved => "saved",
                SaveStatus::Saving => "saving",
                SaveStatus::Failed => "failed",
            }
            .to_string(),
            backup_health: health.label,
            backup_detail: health.detail,
            notification,
            recovery: self.recovery.as_ref().map(|recovery| FrameRecovery {
                reason: recovery.reason.clone(),
                can_restore_previous: recovery.previous_snapshot.is_some(),
            }),
            confirm,
            suggested_export_name: backup_file_name(Utc::now()),
            data_directory: self.store.data_dir().to_path_buf(),
            layout: self.app.layout,
            list_title: list.title,
            list_body: list.body,
            drawer_open: self.app.note_list_visible,
            editing_tags: self.app.is_editing_tags(),
            tag_cleanup,
            quarantine: if self.store.has_quarantined_corrupt_payloads() {
                "Corrupt payload quarantined".to_string()
            } else {
                "No corrupt payload quarantine".to_string()
            },
            focus_title: self.app.workspace.focus_intent() == FocusIntent::NoteTitle,
        }
    }
}

enum ConfirmKind {
    Delete(Uuid),
    Permanent(Uuid),
    Clear,
    Import,
}

fn allowed_during_recovery(command: &Command) -> bool {
    matches!(
        command,
        Command::Snapshot {}
            | Command::Flush {}
            | Command::RestorePrevious {}
            | Command::StartEmpty {}
            | Command::ImportNotebook { .. }
            | Command::AcceptConfirm {}
            | Command::DismissConfirm {}
            | Command::Preview { .. }
            | Command::Format { .. }
            | Command::ExternalNavigation { .. }
            | Command::Resize { .. }
            | Command::DismissNotification { .. }
    )
}

fn selection_bytes(content: &str, selection: TextSelection) -> Result<ByteSelection, SessionError> {
    let (start, end) = match selection {
        TextSelection::Utf8 { start, end } => (start, end),
        TextSelection::Utf16 { start, end } => {
            (utf16_to_byte(content, start)?, utf16_to_byte(content, end)?)
        }
    };
    ByteSelection::new(content, start, end)
        .ok_or_else(|| SessionError::new("invalid_selection", "Invalid selection"))
}

fn load_profile(
    store: &NativeStore,
) -> Result<(AppModel, Option<NativeRecovery>, Preferences), StorageError> {
    let (collection, recovery) = match store.load_collection()? {
        LoadOutcome::Ready(collection) => (collection, None),
        LoadOutcome::Recovery(recovery) => (CollectionEnvelope::empty(), Some(recovery)),
    };
    let preferences = store.load_preferences();
    let mut app = AppModel::new(collection, preferences.theme, store.load_backup_health());
    app.set_storage_recovery(recovery.is_some());
    Ok((app, recovery, preferences))
}

fn resolve_recovery(
    app: &mut AppModel,
    store: &NativeStore,
    recovery: &mut Option<NativeRecovery>,
    empty: bool,
) -> Result<(), StorageError> {
    let state = recovery
        .as_ref()
        .ok_or_else(|| StorageError::InvalidCollection("No recovery is pending".into()))?;
    let collection = if empty {
        store.start_empty(state, Utc::now())?.0
    } else {
        store.restore_previous(state)?
    };
    app.replace_loaded_collection(collection);
    app.set_storage_recovery(false);
    *recovery = None;
    Ok(())
}

fn exchange_error(error: ExchangeError) -> SessionError {
    let code = match &error {
        ExchangeError::RecoveryRefusesCompleteRestore => "storage_recovery",
        ExchangeError::Transition(_) => "invalid_transition",
        ExchangeError::Backup(_)
        | ExchangeError::Unreadable(_)
        | ExchangeError::UnsupportedKind(_) => "invalid_backup",
    };
    SessionError::new(code, error)
}

fn utf16_to_byte(content: &str, offset: usize) -> Result<usize, SessionError> {
    let mut units = 0;
    for (byte, character) in content.char_indices() {
        if units == offset {
            return Ok(byte);
        }
        units += character.len_utf16();
    }
    if units == offset {
        Ok(content.len())
    } else {
        Err(SessionError::new(
            "invalid_selection",
            "Selection splits a surrogate pair or exceeds the content",
        ))
    }
}
