use crate::model::Note;
use crate::note_collection::NoteCollection;
use crate::note_discovery::{NoteListProjection, project_note_list};
use uuid::Uuid;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoteWorkspace {
    notes: Vec<Note>,
    selected_id: Option<Uuid>,
    focus_intent: FocusIntent,
    delete_confirmation: Option<DeleteConfirmation>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FocusIntent {
    #[default]
    None,
    NoteTitle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeleteConfirmation {
    id: Uuid,
    title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDisplayState {
    EmptyCollection,
    NoNoteSelected,
    NoteSelected,
}

impl NoteWorkspace {
    pub fn new(notes: Vec<Note>) -> Self {
        let selected_id = notes.first().map(|note| note.id);
        Self {
            notes,
            selected_id,
            focus_intent: FocusIntent::None,
            delete_confirmation: None,
        }
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected_id
    }

    pub fn selected_note(&self) -> Option<Note> {
        self.selected_note_ref().cloned()
    }

    pub fn display_state(&self) -> WorkspaceDisplayState {
        if self.notes.is_empty() {
            return WorkspaceDisplayState::EmptyCollection;
        }

        if self.selected_note_ref().is_some() {
            WorkspaceDisplayState::NoteSelected
        } else {
            WorkspaceDisplayState::NoNoteSelected
        }
    }

    pub fn create_note(&mut self) {
        let created = NoteCollection::create_note(&mut self.notes);
        self.selected_id = created.selected_id;
        self.focus_intent = if created.should_focus_title {
            FocusIntent::NoteTitle
        } else {
            FocusIntent::None
        };
    }

    pub fn select_note(&mut self, id: Uuid) -> bool {
        if self.notes.iter().any(|note| note.id == id) {
            self.selected_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn update_selected_title(&mut self, title: String) -> bool {
        let Some(id) = self.selected_id else {
            return false;
        };
        NoteCollection::update_title(&mut self.notes, id, title)
    }

    pub fn update_selected_content(&mut self, content: String) -> bool {
        let Some(id) = self.selected_id else {
            return false;
        };
        NoteCollection::update_content(&mut self.notes, id, content)
    }

    pub fn update_selected_tags(&mut self, tags: Vec<String>) -> bool {
        let Some(id) = self.selected_id else {
            return false;
        };
        NoteCollection::update_tags(&mut self.notes, id, tags)
    }

    pub fn request_delete(&mut self, id: Uuid) -> bool {
        let Some(note) = self.notes.iter().find(|note| note.id == id) else {
            return false;
        };

        self.selected_id = Some(id);
        self.delete_confirmation = Some(DeleteConfirmation {
            id,
            title: note.display_title().to_string(),
        });
        true
    }

    pub fn cancel_delete(&mut self) {
        self.delete_confirmation = None;
    }

    pub fn is_delete_confirmation_open(&self) -> bool {
        self.delete_confirmation.is_some()
    }

    pub fn delete_confirmation_title(&self) -> Option<&str> {
        self.delete_confirmation
            .as_ref()
            .map(|confirmation| confirmation.title.as_str())
    }

    pub fn confirm_delete(&mut self) -> bool {
        let Some(confirmation) = self.delete_confirmation.take() else {
            return false;
        };

        if self.notes.iter().any(|note| note.id == confirmation.id) {
            self.selected_id = NoteCollection::delete_note(&mut self.notes, confirmation.id);
            true
        } else {
            false
        }
    }

    pub fn toggle_pin(&mut self, id: Uuid) -> bool {
        NoteCollection::toggle_pin(&mut self.notes, id)
    }

    pub fn focus_intent(&self) -> FocusIntent {
        self.focus_intent
    }

    pub fn take_focus_intent(&mut self) -> FocusIntent {
        let intent = self.focus_intent;
        self.focus_intent = FocusIntent::None;
        intent
    }

    pub fn note_list_projection(
        &self,
        query: &str,
        active_tag: Option<&str>,
    ) -> NoteListProjection {
        project_note_list(&self.notes, self.selected_id, query, active_tag)
    }

    fn selected_note_ref(&self) -> Option<&Note> {
        self.selected_id
            .and_then(|id| self.notes.iter().find(|note| note.id == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_a_note_updates_selection_and_focus_intent_inside_the_workspace() {
        let mut workspace = NoteWorkspace::new(Vec::new());

        workspace.create_note();

        assert_eq!(workspace.notes().len(), 1);
        assert_eq!(workspace.selected_id(), Some(workspace.notes()[0].id));
        assert_eq!(workspace.take_focus_intent(), FocusIntent::NoteTitle);
        assert_eq!(workspace.take_focus_intent(), FocusIntent::None);
    }

    #[test]
    fn distinguishes_empty_collection_from_a_selected_note() {
        assert_eq!(
            NoteWorkspace::new(Vec::new()).display_state(),
            WorkspaceDisplayState::EmptyCollection
        );

        let note = Note::new("First".to_string(), String::new());
        assert_eq!(
            NoteWorkspace::new(vec![note]).display_state(),
            WorkspaceDisplayState::NoteSelected
        );
    }

    #[test]
    fn selected_display_state_does_not_change_when_selected_note_content_changes() {
        let note = Note::new("First".to_string(), "Draft".to_string());
        let mut workspace = NoteWorkspace::new(vec![note]);
        let before = workspace.display_state();

        assert!(workspace.update_selected_title("Updated".to_string()));
        assert!(workspace.update_selected_content("Updated draft".to_string()));
        assert!(workspace.update_selected_tags(vec!["work".to_string()]));
        let after = workspace.display_state();

        assert_eq!(before, after);
    }

    #[test]
    fn creates_selects_and_updates_the_selected_note() {
        let mut workspace = NoteWorkspace::new(Vec::new());
        workspace.create_note();
        let selected_id = workspace.selected_id();

        assert_eq!(
            workspace.selected_note().unwrap().id,
            workspace.notes()[0].id
        );
        assert_eq!(workspace.take_focus_intent(), FocusIntent::NoteTitle);

        assert!(workspace.update_selected_title("Title".to_string()));
        assert!(workspace.update_selected_content("Content".to_string()));
        assert!(workspace.update_selected_tags(vec!["work".to_string()]));

        let selected = workspace.selected_note().unwrap();
        assert_eq!(selected_id, Some(selected.id));
        assert_eq!(selected.title, "Title");
        assert_eq!(selected.content, "Content");
        assert_eq!(selected.tags, vec!["work".to_string()]);
    }

    #[test]
    fn delete_confirmation_selects_deletes_and_clears_confirmation() {
        let first = Note::new("First".to_string(), String::new());
        let second = Note::new("Second".to_string(), String::new());
        let first_id = first.id;
        let second_id = second.id;
        let mut workspace = NoteWorkspace::new(vec![first, second]);

        assert!(workspace.request_delete(first_id));
        assert_eq!(workspace.selected_id(), Some(first_id));
        assert!(workspace.is_delete_confirmation_open());

        assert!(workspace.confirm_delete());
        assert_eq!(workspace.selected_id(), Some(second_id));
        assert!(!workspace.is_delete_confirmation_open());
        assert_eq!(workspace.notes().len(), 1);
        assert_eq!(workspace.notes()[0].id, second_id);
    }

    #[test]
    fn delete_confirmation_identifies_the_selected_note() {
        let note = Note::new("Delete me".to_string(), String::new());
        let note_id = note.id;
        let mut workspace = NoteWorkspace::new(vec![note]);
        assert!(workspace.request_delete(note_id));

        assert_eq!(workspace.delete_confirmation_title(), Some("Delete me"));
    }

    #[test]
    fn cancelled_delete_leaves_notes_unchanged() {
        let note = Note::new("First".to_string(), String::new());
        let note_id = note.id;
        let mut workspace = NoteWorkspace::new(vec![note]);

        assert!(workspace.request_delete(note_id));
        workspace.cancel_delete();

        assert!(!workspace.is_delete_confirmation_open());
        assert_eq!(workspace.notes().len(), 1);
        assert_eq!(workspace.notes()[0].id, note_id);
        assert!(!workspace.confirm_delete());
        assert_eq!(workspace.notes().len(), 1);
    }

    #[test]
    fn confirmed_delete_uses_the_confirmation_target_not_later_selection() {
        let first = Note::new("First".to_string(), String::new());
        let second = Note::new("Second".to_string(), String::new());
        let first_id = first.id;
        let second_id = second.id;
        let mut workspace = NoteWorkspace::new(vec![first, second]);

        assert!(workspace.request_delete(first_id));
        assert!(workspace.select_note(second_id));
        assert!(workspace.confirm_delete());

        assert_eq!(workspace.notes().len(), 1);
        assert_eq!(workspace.notes()[0].id, second_id);
    }

    #[test]
    fn pinning_changes_note_order_in_projection() {
        let first = Note::new("First".to_string(), String::new());
        let second = Note::new("Second".to_string(), String::new());
        let second_id = second.id;
        let mut workspace = NoteWorkspace::new(vec![first, second]);

        assert!(workspace.toggle_pin(second_id));
        assert!(workspace.notes()[1].is_pinned);
        assert_eq!(
            workspace.note_list_projection("", None).rows[0].id,
            second_id
        );
    }
}
