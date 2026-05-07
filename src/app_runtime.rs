use crate::AppState;
use crate::backup::BackupHealthRecord;
use crate::editor_view::EditorViewMode;
use crate::model::Note;
use crate::note_list_interaction::NoteListInteraction;
use crate::note_workspace::NoteWorkspace;
use crate::responsive_navigation::{
    NoteListPersistence, ResponsiveNavigation, StoredNoteListState, ViewportClass,
    normalize_view_mode,
};
use crate::storage::{
    SaveSession, SaveStatus, save_dark_mode, save_recently_deleted_notes, save_sidebar_open,
};
use leptos::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct AppRuntimeStartup {
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
    pub is_dark_mode: bool,
    pub viewport_class: ViewportClass,
    pub stored_note_list_state: StoredNoteListState,
    pub backup_health_record: Option<BackupHealthRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePersistenceSnapshot {
    pub is_dark_mode: bool,
    pub viewport_class: ViewportClass,
    pub note_list_state_to_persist: Option<StoredNoteListState>,
    pub notes_save_revision: u64,
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
    pub editor_view_mode: EditorViewMode,
    pub backup_health_record: Option<BackupHealthRecord>,
    pub save_status: SaveStatus,
}

impl AppState {
    pub fn from_startup(startup: AppRuntimeStartup) -> Self {
        let initial_navigation =
            ResponsiveNavigation::initial(startup.viewport_class, startup.stored_note_list_state);

        Self {
            workspace: RwSignal::new(NoteWorkspace::new_with_recently_deleted(
                startup.notes,
                startup.recently_deleted_notes,
            )),
            notes_save_revision: RwSignal::new(0),
            is_dark_mode: RwSignal::new(startup.is_dark_mode),
            viewport_class: RwSignal::new(startup.viewport_class),
            is_sidebar_open: RwSignal::new(initial_navigation.is_note_list_visible()),
            note_list_interaction: RwSignal::new(NoteListInteraction::default()),
            editor_view_mode: RwSignal::new(EditorViewMode::Write),
            save_status: RwSignal::new(SaveStatus::Saved),
            backup_health_record: RwSignal::new(startup.backup_health_record),
            notification: RwSignal::new(None),
            notification_sequence: RwSignal::new(0),
        }
    }

    pub fn persistence_snapshot(self) -> RuntimePersistenceSnapshot {
        self.persistence_snapshot_with_revision(self.notes_save_revision.get_untracked())
    }

    pub fn tracked_notes_persistence_snapshot(self) -> RuntimePersistenceSnapshot {
        let revision = self.notes_save_revision.get();
        self.persistence_snapshot_with_revision(revision)
    }

    pub fn note_list_state_to_persist(self) -> Option<StoredNoteListState> {
        let navigation =
            ResponsiveNavigation::current(self.viewport_class.get(), self.is_sidebar_open.get());
        match navigation.persistence() {
            NoteListPersistence::Persist(stored_state) => Some(stored_state),
            NoteListPersistence::Skip => None,
        }
    }

    pub fn reclassify_viewport(self, next_viewport_class: ViewportClass) -> bool {
        if self.viewport_class.get_untracked() == next_viewport_class {
            return false;
        }

        let mut navigation = self.responsive_navigation_untracked();
        navigation.reclassify_viewport(next_viewport_class);

        self.viewport_class.set(next_viewport_class);
        self.is_sidebar_open.set(navigation.is_note_list_visible());
        self.editor_view_mode.update(|view_mode| {
            *view_mode = normalize_view_mode(next_viewport_class, *view_mode);
        });
        true
    }

    fn persistence_snapshot_with_revision(
        self,
        notes_save_revision: u64,
    ) -> RuntimePersistenceSnapshot {
        RuntimePersistenceSnapshot {
            is_dark_mode: self.is_dark_mode.get_untracked(),
            viewport_class: self.viewport_class.get_untracked(),
            note_list_state_to_persist: self.note_list_state_to_persist_untracked(),
            notes_save_revision,
            notes: self.notes_untracked(),
            recently_deleted_notes: self.recently_deleted_notes_untracked(),
            editor_view_mode: self.editor_view_mode.get_untracked(),
            backup_health_record: self.backup_health_record.get_untracked(),
            save_status: self.save_status.get_untracked(),
        }
    }

    fn note_list_state_to_persist_untracked(self) -> Option<StoredNoteListState> {
        let navigation = ResponsiveNavigation::current(
            self.viewport_class.get_untracked(),
            self.is_sidebar_open.get_untracked(),
        );
        match navigation.persistence() {
            NoteListPersistence::Persist(stored_state) => Some(stored_state),
            NoteListPersistence::Skip => None,
        }
    }
}

pub fn install_runtime_persistence(state: AppState) -> SaveSession {
    Effect::new(move |_| {
        save_dark_mode(state.is_dark_mode.get());
    });

    Effect::new(move |_| {
        if let Some(stored_state) = state.note_list_state_to_persist() {
            save_sidebar_open(stored_state.is_open());
        }
    });

    let save_session = SaveSession::default();
    let save_session_for_effect = save_session.clone();
    let is_initial_notes_effect = Rc::new(Cell::new(true));
    Effect::new(move |_| {
        let snapshot = state.tracked_notes_persistence_snapshot();
        if is_initial_notes_effect.replace(false) {
            return;
        }
        save_session_for_effect.schedule_notes_save(snapshot.notes, state.save_status);
        save_recently_deleted_notes(&snapshot.recently_deleted_notes);
    });

    save_session.install_page_flush_listeners(
        move || {
            let snapshot = state.persistence_snapshot();
            save_recently_deleted_notes(&snapshot.recently_deleted_notes);
            snapshot.notes
        },
        state.save_status,
    );

    save_session
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::BackupHealthRecord;
    use crate::editor_view::EditorViewMode;
    use crate::model::Note;
    use crate::responsive_navigation::{StoredNoteListState, ViewportClass};
    use crate::storage::SaveStatus;
    use chrono::Utc;
    use leptos::prelude::Owner;

    #[test]
    fn runtime_startup_exposes_persistence_snapshot_without_raw_signal_access() {
        let active_note = Note::new("Active".to_string(), "Keep writing".to_string());
        let deleted_note = Note::new("Deleted".to_string(), "Recover me".to_string());
        let backup_health_record = BackupHealthRecord {
            last_successful_export_at: Utc::now(),
        };

        Owner::new().with(|| {
            let state = crate::AppState::from_startup(AppRuntimeStartup {
                notes: vec![active_note.clone()],
                recently_deleted_notes: vec![deleted_note.clone()],
                is_dark_mode: true,
                viewport_class: ViewportClass::Compact,
                stored_note_list_state: StoredNoteListState::Closed,
                backup_health_record: Some(backup_health_record),
            });

            let snapshot = state.persistence_snapshot();

            assert!(snapshot.is_dark_mode);
            assert_eq!(
                snapshot.note_list_state_to_persist,
                Some(StoredNoteListState::Closed)
            );
            assert_eq!(snapshot.notes, vec![active_note]);
            assert_eq!(snapshot.recently_deleted_notes, vec![deleted_note]);
            assert_eq!(snapshot.backup_health_record, Some(backup_health_record));
            assert_eq!(snapshot.save_status, SaveStatus::Saved);
        });
    }

    #[test]
    fn runtime_reclassifies_viewport_and_normalises_view_mode() {
        Owner::new().with(|| {
            let state = crate::AppState::from_startup(AppRuntimeStartup {
                notes: Vec::new(),
                recently_deleted_notes: Vec::new(),
                is_dark_mode: false,
                viewport_class: ViewportClass::Wide,
                stored_note_list_state: StoredNoteListState::Closed,
                backup_health_record: None,
            });

            state.set_editor_view_mode(EditorViewMode::Split);

            assert!(state.reclassify_viewport(ViewportClass::Compact));

            let snapshot = state.persistence_snapshot();
            assert_eq!(snapshot.viewport_class, ViewportClass::Compact);
            assert_eq!(snapshot.editor_view_mode, EditorViewMode::Write);
            assert_eq!(
                snapshot.note_list_state_to_persist,
                Some(StoredNoteListState::Open)
            );

            assert!(!state.reclassify_viewport(ViewportClass::Compact));
        });
    }
}
