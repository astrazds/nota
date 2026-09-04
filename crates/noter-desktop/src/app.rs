use chrono::{DateTime, Utc};
use noter_core::NoteWorkspace;
use noter_core::backup::{BackupHealthRecord, import_flat_collection_backup};
use noter_core::editor_view::EditorViewMode;
use noter_core::markdown_editing::{ByteSelection, MarkdownCommand, apply_markdown_command};
use noter_core::note_list_interaction::{NoteListInteraction, NoteListRenderModel};
use noter_core::responsive_navigation::{ViewportClass, normalize_view_mode};
use noter_core::tag_rules::parse_tags_input;
use noter_core::transition::{ThemePreference, TransitionError, import_desktop_transition};
use uuid::Uuid;

use crate::storage::CollectionEnvelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Saved,
    Saving,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationTone {
    Progress,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalNotification {
    pub message: String,
    pub tone: NotificationTone,
}

#[derive(Debug)]
pub enum AppMsg {
    QuickCapture,
    SelectNote(Uuid),
    UpdateTitle(String),
    UpdateContent(String),
    UpdateTags(String),
    ApplyFormatting {
        selection: ByteSelection,
        command: MarkdownCommand,
    },
    EditSearch(String),
    CommitSearch,
    SelectTag(String),
    ClearTag,
    TogglePin(Uuid),
    RequestDelete(Uuid),
    CancelDelete,
    ConfirmDelete,
    RestoreRecentlyDeleted(Uuid),
    PermanentlyDelete(Uuid),
    RequestClearAll,
    CancelClearAll,
    ConfirmClearAll,
    SetViewMode(EditorViewMode),
    Resize(f64),
    ToggleNavigation,
    RestorePreviousSnapshot,
    StartEmptyAfterRecovery,
    FlushPersistence,
    PersistenceComplete(u64),
    PersistenceFailed(String),
    RequestBackupExport,
    RequestBackupImport,
    RequestTransitionExport,
    RequestTransitionImport,
    RequestDiagnostics,
    ImportBackupJson(String),
    ImportTransitionJson(String),
    BackupExported(DateTime<Utc>),
    ToggleTheme,
    OperationSucceeded(String),
    OperationFailed(String),
}

#[derive(Debug)]
pub struct AppModel {
    pub workspace: NoteWorkspace,
    pub note_list: NoteListInteraction,
    pub theme: ThemePreference,
    pub view_mode: EditorViewMode,
    pub viewport: ViewportClass,
    pub note_list_visible: bool,
    pub save_status: SaveStatus,
    pub notification: Option<GlobalNotification>,
    pub backup_health: Option<BackupHealthRecord>,
    revision: u64,
}

impl AppModel {
    pub fn new(
        collection: CollectionEnvelope,
        theme: ThemePreference,
        backup_health: Option<BackupHealthRecord>,
    ) -> Self {
        Self {
            workspace: NoteWorkspace::new_with_recently_deleted(
                collection.notes,
                collection.recently_deleted_notes,
            ),
            note_list: NoteListInteraction::default(),
            theme,
            view_mode: EditorViewMode::Write,
            viewport: ViewportClass::Wide,
            note_list_visible: true,
            save_status: SaveStatus::Saved,
            notification: None,
            backup_health,
            revision: 0,
        }
    }

    pub fn apply(&mut self, message: AppMsg) -> bool {
        let changed = match message {
            AppMsg::QuickCapture => {
                self.workspace.create_note();
                true
            }
            AppMsg::SelectNote(id) => {
                let selected = self.workspace.select_note(id);
                if selected && self.viewport == ViewportClass::Compact {
                    self.note_list_visible = false;
                }
                false
            }
            AppMsg::UpdateTitle(title) => self.workspace.update_selected_title(title),
            AppMsg::UpdateContent(content) => self.workspace.update_selected_content(content),
            AppMsg::UpdateTags(tags) => {
                self.workspace.update_selected_tags(parse_tags_input(&tags))
            }
            AppMsg::ApplyFormatting { selection, command } => {
                let Some(note) = self.workspace.selected_note() else {
                    return false;
                };
                let formatted = apply_markdown_command(&note.content, selection, command);
                self.workspace.update_selected_content(formatted.content)
            }
            AppMsg::EditSearch(search) => {
                self.note_list.edit_search(search);
                false
            }
            AppMsg::CommitSearch => {
                self.note_list.commit_search();
                false
            }
            AppMsg::SelectTag(tag) => {
                self.note_list.select_tag(tag);
                false
            }
            AppMsg::ClearTag => {
                self.note_list.clear_tag();
                false
            }
            AppMsg::TogglePin(id) => self.workspace.toggle_pin(id),
            AppMsg::RequestDelete(id) => self.workspace.request_delete(id) && false,
            AppMsg::CancelDelete => {
                self.workspace.cancel_delete();
                false
            }
            AppMsg::ConfirmDelete => self.workspace.confirm_delete(),
            AppMsg::RestoreRecentlyDeleted(id) => self.workspace.restore_recently_deleted(id),
            AppMsg::PermanentlyDelete(id) => self.workspace.permanently_clear_recently_deleted(id),
            AppMsg::RequestClearAll => self.workspace.request_clear_all_recently_deleted() && false,
            AppMsg::CancelClearAll => {
                self.workspace.cancel_clear_all_recently_deleted();
                false
            }
            AppMsg::ConfirmClearAll => self.workspace.confirm_clear_all_recently_deleted(),
            AppMsg::SetViewMode(mode) => {
                self.view_mode = normalize_view_mode(self.viewport, mode);
                false
            }
            AppMsg::Resize(width) => {
                // GTK may notify width=0 before the window is mapped. Treating that
                // as Compact hides the sidebar/editor exclusively and sticks there.
                if width <= 0.0 {
                    return false;
                }
                self.viewport = ViewportClass::from_width(width);
                if self.viewport == ViewportClass::Wide {
                    self.note_list_visible = true;
                }
                self.view_mode = normalize_view_mode(self.viewport, self.view_mode);
                false
            }
            AppMsg::ToggleNavigation => {
                if self.viewport == ViewportClass::Compact {
                    self.note_list_visible = !self.note_list_visible;
                }
                false
            }
            AppMsg::RestorePreviousSnapshot | AppMsg::StartEmptyAfterRecovery => false,
            AppMsg::FlushPersistence => false,
            AppMsg::PersistenceComplete(revision) => {
                if revision == self.revision {
                    self.save_status = SaveStatus::Saved;
                }
                false
            }
            AppMsg::PersistenceFailed(error) => {
                self.save_status = SaveStatus::Failed;
                self.notification = Some(GlobalNotification {
                    message: error,
                    tone: NotificationTone::Error,
                });
                false
            }
            AppMsg::BackupExported(exported_at) => {
                self.backup_health = Some(BackupHealthRecord {
                    last_successful_export_at: exported_at,
                });
                self.notification = Some(GlobalNotification {
                    message: "Backup exported".to_string(),
                    tone: NotificationTone::Success,
                });
                false
            }
            AppMsg::ToggleTheme => {
                self.theme = match self.theme {
                    ThemePreference::Dark => ThemePreference::Light,
                    ThemePreference::Light | ThemePreference::System => ThemePreference::Dark,
                };
                false
            }
            AppMsg::OperationSucceeded(message) => {
                self.notification = Some(GlobalNotification {
                    message,
                    tone: NotificationTone::Success,
                });
                false
            }
            AppMsg::OperationFailed(message) => {
                self.notification = Some(GlobalNotification {
                    message,
                    tone: NotificationTone::Error,
                });
                false
            }
            AppMsg::RequestBackupExport
            | AppMsg::RequestBackupImport
            | AppMsg::RequestTransitionExport
            | AppMsg::RequestTransitionImport
            | AppMsg::RequestDiagnostics
            | AppMsg::ImportBackupJson(_)
            | AppMsg::ImportTransitionJson(_) => false,
        };

        if changed {
            self.revision = self.revision.saturating_add(1);
            self.save_status = SaveStatus::Saving;
        }
        changed
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn collection(&self) -> CollectionEnvelope {
        CollectionEnvelope::new(
            self.workspace.notes().to_vec(),
            self.workspace.recently_deleted_notes().to_vec(),
        )
    }

    pub fn note_list_render_model(&self) -> NoteListRenderModel {
        self.note_list
            .render_model(self.workspace.notes(), self.workspace.selected_id())
    }

    pub fn replace_loaded_collection(&mut self, collection: CollectionEnvelope) {
        self.workspace = NoteWorkspace::new_with_recently_deleted(
            collection.notes,
            collection.recently_deleted_notes,
        );
        self.save_status = SaveStatus::Saved;
        self.notification = None;
    }

    pub fn import_backup(&mut self, json: &str) -> Result<(), noter_core::backup::BackupError> {
        let mut notes = self.workspace.notes().to_vec();
        let deleted = self.workspace.recently_deleted_notes().to_vec();
        let imported = import_flat_collection_backup(&mut notes, json)?;
        self.workspace = NoteWorkspace::new_with_recently_deleted(notes, deleted);
        if let Some(id) = imported.selected_id {
            self.workspace.select_note(id);
        }
        self.mark_external_change("Backup imported");
        Ok(())
    }

    pub fn import_transition(&mut self, json: &str) -> Result<(), TransitionError> {
        let restored = import_desktop_transition(
            self.workspace.notes(),
            self.workspace.recently_deleted_notes(),
            json,
        )?;
        self.workspace = NoteWorkspace::new_with_recently_deleted(
            restored.notes,
            restored.recently_deleted_notes,
        );
        self.theme = restored.theme;
        self.backup_health = restored.backup_health;
        self.mark_external_change("Desktop transition restored");
        Ok(())
    }

    fn mark_external_change(&mut self, message: &str) {
        self.revision = self.revision.saturating_add(1);
        self.save_status = SaveStatus::Saving;
        self.notification = Some(GlobalNotification {
            message: message.to_string(),
            tone: NotificationTone::Success,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noter_core::transition::export_desktop_transition;

    #[test]
    fn quick_capture_edit_search_delete_restore_and_clear_follow_messages() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
        assert!(app.apply(AppMsg::QuickCapture));
        let id = app.workspace.selected_id().unwrap();
        assert!(app.apply(AppMsg::UpdateTitle("Roadmap".to_string())));
        assert!(app.apply(AppMsg::UpdateContent("Ship native app".to_string())));
        assert!(app.apply(AppMsg::UpdateTags("Work, Rust".to_string())));

        app.apply(AppMsg::EditSearch("native".to_string()));
        app.apply(AppMsg::CommitSearch);
        assert_eq!(app.note_list_render_model().projection.rows.len(), 1);

        app.apply(AppMsg::RequestDelete(id));
        assert!(app.apply(AppMsg::ConfirmDelete));
        assert!(app.workspace.notes().is_empty());
        assert_eq!(app.workspace.recently_deleted_notes().len(), 1);

        assert!(app.apply(AppMsg::RestoreRecentlyDeleted(id)));
        app.apply(AppMsg::RequestDelete(id));
        app.apply(AppMsg::ConfirmDelete);
        app.apply(AppMsg::RequestClearAll);
        assert!(app.apply(AppMsg::ConfirmClearAll));
        assert!(app.workspace.recently_deleted_notes().is_empty());
    }

    #[test]
    fn resize_ignores_non_positive_widths_so_unmapped_windows_stay_dual_pane() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        assert_eq!(app.viewport, ViewportClass::Wide);
        assert!(app.note_list_visible);

        app.apply(AppMsg::Resize(0.0));
        assert_eq!(app.viewport, ViewportClass::Wide);
        assert!(app.note_list_visible);

        app.apply(AppMsg::Resize(-1.0));
        assert_eq!(app.viewport, ViewportClass::Wide);
    }

    #[test]
    fn split_view_normalizes_when_the_window_becomes_narrow() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        app.apply(AppMsg::SetViewMode(EditorViewMode::Split));
        assert_eq!(app.view_mode, EditorViewMode::Split);

        app.apply(AppMsg::Resize(600.0));

        assert_eq!(app.viewport, ViewportClass::Compact);
        assert_eq!(app.view_mode, EditorViewMode::Write);
    }

    #[test]
    fn compact_navigation_uses_full_width_surfaces_and_selection_returns_to_writing() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
        app.apply(AppMsg::QuickCapture);
        let id = app.workspace.selected_id().unwrap();
        app.apply(AppMsg::Resize(700.0));

        app.apply(AppMsg::ToggleNavigation);
        assert!(!app.note_list_visible);
        app.apply(AppMsg::ToggleNavigation);
        assert!(app.note_list_visible);

        app.apply(AppMsg::SelectNote(id));
        assert!(!app.note_list_visible);
        app.apply(AppMsg::Resize(1200.0));
        assert!(app.note_list_visible);
    }

    #[test]
    fn theme_changes_do_not_create_a_collection_revision() {
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);

        app.apply(AppMsg::ToggleTheme);

        assert_eq!(app.theme, ThemePreference::Dark);
        assert_eq!(app.revision(), 0);
        assert_eq!(app.save_status, SaveStatus::Saved);
    }

    #[test]
    fn transition_restore_is_exact_on_first_run_and_rejected_after_native_data_exists() {
        let imported = noter_core::Note::new("From web".to_string(), "Exact state".to_string());
        let json = export_desktop_transition(
            std::slice::from_ref(&imported),
            &[],
            ThemePreference::Dark,
            None,
        )
        .unwrap();
        let mut first_run =
            AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);

        first_run.import_transition(&json).unwrap();

        assert_eq!(first_run.workspace.notes(), std::slice::from_ref(&imported));
        assert_eq!(first_run.theme, ThemePreference::Dark);
        assert_eq!(first_run.revision(), 1);

        let before = first_run.collection();
        let revision = first_run.revision();
        let error = first_run
            .import_transition(&json)
            .expect_err("native data must block an exact transition restore");
        assert!(matches!(error, TransitionError::CollectionNotEmpty));
        assert_eq!(first_run.collection(), before);
        assert_eq!(first_run.revision(), revision);
    }
}
