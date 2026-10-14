use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use chrono::Utc;
use nota_core::backup::{BackupHealthRecord, backup_file_name};
use nota_core::editor_view::EditorViewMode;
use nota_core::exchange::{ExchangeError, ImportDestination, export_notebook, stage_import};
use nota_core::markdown_editing::{ByteSelection, MarkdownCommand, apply_markdown_command};
use nota_core::tag_rules::collect_note_tags;
use nota_core::transition::ThemePreference;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::app::{AppModel, AppMsg, AppliedImport, NotificationTone, SaveStatus};
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

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Snapshot,
    NewNote,
    Flush,
    SelectNote {
        id: Uuid,
    },
    EditNote {
        id: Uuid,
        edit_sequence: u64,
        title: String,
        content: String,
        tags_input: String,
    },
    Search {
        query: String,
    },
    FilterTag {
        tag: Option<String>,
    },
    FilterPinned {
        pinned: bool,
    },
    TogglePin {
        id: Uuid,
    },
    DeleteNote {
        id: Uuid,
    },
    RestoreNote {
        id: Uuid,
    },
    PermanentlyDelete {
        id: Uuid,
    },
    ClearDeleted,
    SetTheme {
        theme: ThemePreference,
    },
    SetViewMode {
        mode: ViewMode,
    },
    Format {
        content: String,
        start_utf16: usize,
        end_utf16: usize,
        kind: FormatKind,
    },
    Preview {
        title: String,
        content: String,
        dark: bool,
        layout: PreviewLayout,
    },
    ImportNotebook {
        json: String,
    },
    ConfirmImport,
    CancelImport,
    ExportNotebook {
        path: PathBuf,
    },
    RestorePrevious,
    StartEmpty,
    ExternalNavigation {
        uri: String,
        user_activated: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Write,
    Preview,
    Split,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatKind {
    Bold,
    Italic,
    Heading,
    BulletList,
    Link,
    Strikethrough,
    TaskList,
    Table,
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
            _profile_lock: profile_lock,
        })
    }

    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    /// GTK messages use the same recovery and save lifecycle as ABI commands.
    pub fn apply_message(&mut self, message: AppMsg) -> Result<bool, SessionError> {
        self.poll();
        match message {
            AppMsg::RestorePreviousSnapshot => {
                resolve_recovery(&mut self.app, &self.store, &mut self.recovery, false)?;
                return Ok(false);
            }
            AppMsg::StartEmptyAfterRecovery => {
                resolve_recovery(&mut self.app, &self.store, &mut self.recovery, true)?;
                return Ok(false);
            }
            AppMsg::ConfirmBackupImport => {
                self.execute(Request::ConfirmImport)?;
                return Ok(true);
            }
            AppMsg::CancelBackupImport => {
                self.execute(Request::CancelImport)?;
                return Ok(false);
            }
            AppMsg::PollPersistence => return Ok(false),
            _ => {}
        }
        let changed = self.app.apply(message);
        if changed {
            self.schedule_save()?;
        }
        Ok(changed)
    }

    fn schedule_save(&mut self) -> Result<(), SessionError> {
        if self
            .worker
            .schedule(self.app.revision(), self.app.collection())
        {
            Ok(())
        } else {
            self.app.apply(AppMsg::PersistenceFailed(
                "Persistence worker stopped".into(),
            ));
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
                        self.app.apply(AppMsg::PersistenceFailed(error.to_string()));
                    } else {
                        self.app.apply(AppMsg::PersistenceComplete(revision));
                    }
                }
                Err(error) => {
                    self.app.apply(AppMsg::PersistenceFailed(error.to_string()));
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
                self.app.apply(AppMsg::PersistenceComplete(revision));
                Ok(())
            }
            Err(error) => {
                self.app.apply(AppMsg::PersistenceFailed(error.to_string()));
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

    pub fn execute(&mut self, request: Request) -> Result<Value, SessionError> {
        self.poll();
        if self.recovery.is_some()
            && !matches!(
                &request,
                Request::Snapshot
                    | Request::Flush
                    | Request::RestorePrevious
                    | Request::StartEmpty
                    | Request::ImportNotebook { .. }
                    | Request::ConfirmImport
                    | Request::CancelImport
                    | Request::Preview { .. }
                    | Request::ExternalNavigation { .. }
                    | Request::Format { .. }
            )
        {
            return Err(SessionError::new(
                "storage_recovery",
                "Resolve Storage Recovery before editing",
            ));
        }
        let revision = self.app.revision();
        let result = self.dispatch(request);
        if self.app.revision() != revision {
            self.schedule_save()?;
        }
        result.map(|result| self.reply(result))
    }

    fn dispatch(&mut self, request: Request) -> Result<Value, SessionError> {
        match request {
            Request::Snapshot => {}
            Request::NewNote => {
                self.app.apply(AppMsg::QuickCapture);
            }
            Request::Flush => self.flush()?,
            Request::SelectNote { id } => {
                self.require_note(id, false)?;
                self.app.apply(AppMsg::SelectNote(id));
            }
            Request::EditNote {
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
                self.app.apply(AppMsg::SelectNote(id));
                self.app.apply(AppMsg::UpdateTitle(title));
                self.app.apply(AppMsg::UpdateContent(content));
                self.app.apply(AppMsg::UpdateTags(tags_input));
                if let Some(selected) = selected {
                    self.app.apply(AppMsg::SelectNote(selected));
                }
                self.edit_sequence = edit_sequence;
            }
            Request::Search { query } => {
                self.app.apply(AppMsg::EditSearch(query));
                self.app.apply(AppMsg::CommitSearch);
            }
            Request::FilterTag { tag } => {
                self.app
                    .apply(tag.map_or(AppMsg::ClearTag, AppMsg::SelectTag));
            }
            Request::FilterPinned { pinned } => {
                self.app.apply(AppMsg::SetPinnedFilter(pinned));
            }
            Request::TogglePin { id } => {
                self.require_note(id, false)?;
                self.app.apply(AppMsg::TogglePin(id));
            }
            Request::DeleteNote { id } => {
                self.require_note(id, false)?;
                self.app.apply(AppMsg::RequestDelete(id));
                self.app.apply(AppMsg::ConfirmDelete);
            }
            Request::RestoreNote { id } => {
                self.require_note(id, true)?;
                self.app.apply(AppMsg::RestoreRecentlyDeleted(id));
            }
            Request::PermanentlyDelete { id } => {
                self.require_note(id, true)?;
                self.app.apply(AppMsg::PermanentlyDelete(id));
            }
            Request::ClearDeleted => {
                self.app.apply(AppMsg::RequestClearAll);
                self.app.apply(AppMsg::ConfirmClearAll);
            }
            Request::SetTheme { theme } => {
                let preferences = Preferences {
                    theme,
                    ..self.preferences.clone()
                };
                self.store.save_preferences(&preferences)?;
                self.preferences = preferences;
                self.app.theme = theme;
            }
            Request::SetViewMode { mode } => {
                self.app.apply(AppMsg::SetViewMode(match mode {
                    ViewMode::Write => EditorViewMode::Write,
                    ViewMode::Preview => EditorViewMode::Preview,
                    ViewMode::Split => EditorViewMode::Split,
                }));
            }
            Request::Format {
                content,
                start_utf16,
                end_utf16,
                kind,
            } => {
                let start = utf16_to_byte(&content, start_utf16)?;
                let end = utf16_to_byte(&content, end_utf16)?;
                let selection = ByteSelection::new(&content, start, end)
                    .ok_or_else(|| SessionError::new("invalid_selection", "Invalid selection"))?;
                let command = match kind {
                    FormatKind::Bold => MarkdownCommand::Bold,
                    FormatKind::Italic => MarkdownCommand::Italic,
                    FormatKind::Heading => MarkdownCommand::Heading,
                    FormatKind::BulletList => MarkdownCommand::BulletList,
                    FormatKind::Link => MarkdownCommand::Link,
                    FormatKind::Strikethrough => MarkdownCommand::Strikethrough,
                    FormatKind::TaskList => MarkdownCommand::TaskList,
                    FormatKind::Table => MarkdownCommand::Table,
                };
                let formatted = apply_markdown_command(&content, selection, command);
                let caret_utf16 = formatted.content[..formatted.caret_byte]
                    .encode_utf16()
                    .count();
                return Ok(json!({"content": formatted.content, "caret_utf16": caret_utf16}));
            }
            Request::Preview {
                title,
                content,
                dark,
                layout,
            } => {
                return Ok(json!({"html": preview_document(&title, &content, dark, layout)}));
            }
            Request::ImportNotebook { json } => {
                let staged = {
                    let destination = ImportDestination {
                        notes: self.app.workspace.notes(),
                        recently_deleted: self.app.workspace.recently_deleted_notes(),
                        storage_recovery: self.recovery.is_some(),
                    };
                    stage_import(&json, destination)
                };
                match staged {
                    Ok(staged) => self.app.stage_import(staged),
                    Err(error) => {
                        self.app.clear_pending_import();
                        return Err(exchange_error(error));
                    }
                }
            }
            Request::ConfirmImport => {
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
            }
            Request::CancelImport => {
                self.app.apply(AppMsg::CancelBackupImport);
            }
            Request::ExportNotebook { path } => {
                let exported = self.notebook_export()?;
                write_atomic(&path, exported.json.as_bytes())?;
                self.record_successful_notes_export(exported.health)?;
            }
            Request::RestorePrevious => {
                resolve_recovery(&mut self.app, &self.store, &mut self.recovery, false)?
            }
            Request::StartEmpty => {
                resolve_recovery(&mut self.app, &self.store, &mut self.recovery, true)?
            }
            Request::ExternalNavigation {
                uri,
                user_activated,
            } => {
                return Ok(
                    json!({"uri": external_navigation_target(&uri, user_activated).map(|uri| uri.to_string())}),
                );
            }
        }
        Ok(Value::Null)
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
        self.app
            .apply(AppMsg::BackupExported(health.last_successful_export_at));
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

    pub fn reply(&self, result: Value) -> Value {
        json!({"ok": true, "snapshot": self.snapshot(), "result": result})
    }

    pub fn snapshot(&self) -> Value {
        let rows: Vec<Value> = self.app.note_list_render_model().projection.rows.into_iter().map(|row|
            json!({"id": row.id, "title": row.display_title, "preview": row.preview, "date": row.display_date, "is_pinned": row.is_pinned})
        ).collect();
        let deleted: Vec<Value> = self
            .app
            .workspace
            .recently_deleted_notes()
            .iter()
            .map(|note| json!({"id": note.id, "title": note.display_title()}))
            .collect();
        let notification = self.app.notification.as_ref().map(|notification| json!({
            "message": notification.message, "tone": match notification.tone {
                NotificationTone::Progress => "progress", NotificationTone::Success => "success", NotificationTone::Error => "error"
            }
        }));
        let pending_import = self.app.pending_import().map(|pending| {
            json!({
                "title": pending.title,
                "body": pending.body,
                "action": pending.action,
            })
        });
        json!({
            "revision": self.app.revision(), "edit_sequence": self.edit_sequence,
            "selected_note": self.app.workspace.selected_note(), "rows": rows, "recently_deleted": deleted,
            "search_input": self.app.note_list.search_input(), "active_tag": self.app.note_list.active_tag(),
            "pinned_only": self.app.note_list.pinned_only(),
            "tags": collect_note_tags(self.app.workspace.notes()),
            "view_mode": match self.app.view_mode { EditorViewMode::Write => "write", EditorViewMode::Preview => "preview", EditorViewMode::Split => "split" },
            "theme": self.app.theme,
            "save_status": match self.app.save_status { SaveStatus::Saved => "saved", SaveStatus::Saving => "saving", SaveStatus::Failed => "failed" },
            "backup_health": self.app.backup_health_label(Utc::now()), "notification": notification,
            "recovery": self.recovery.as_ref().map(|recovery| json!({"reason": recovery.reason, "can_restore_previous": recovery.previous_snapshot.is_some()})),
            "suggested_export_name": backup_file_name(Utc::now()),
            "pending_import": pending_import, "data_directory": self.store.data_dir()
        })
    }
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
