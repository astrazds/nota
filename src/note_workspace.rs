use crate::model::Note;
use crate::note_collection::{NoteCollection, NoteCreation};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy)]
pub struct NoteWorkspace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDisplayState {
    EmptyCollection,
    NoNoteSelected,
    NoteSelected(Note),
}

impl NoteWorkspace {
    pub fn selected_note(notes: &[Note], selected_id: Option<Uuid>) -> Option<Note> {
        selected_id.and_then(|id| notes.iter().find(|note| note.id == id).cloned())
    }

    pub fn display_state(notes: &[Note], selected_id: Option<Uuid>) -> WorkspaceDisplayState {
        if notes.is_empty() {
            return WorkspaceDisplayState::EmptyCollection;
        }

        Self::selected_note(notes, selected_id)
            .map(WorkspaceDisplayState::NoteSelected)
            .unwrap_or(WorkspaceDisplayState::NoNoteSelected)
    }

    pub fn create_note(notes: &mut Vec<Note>) -> NoteCreation {
        NoteCollection::create_note(notes)
    }

    pub fn update_selected_title(
        notes: &mut [Note],
        selected_id: Option<Uuid>,
        title: String,
    ) -> bool {
        let Some(id) = selected_id else {
            return false;
        };
        NoteCollection::update_title(notes, id, title)
    }

    pub fn update_selected_content(
        notes: &mut [Note],
        selected_id: Option<Uuid>,
        content: String,
    ) -> bool {
        let Some(id) = selected_id else {
            return false;
        };
        NoteCollection::update_content(notes, id, content)
    }

    pub fn update_selected_tags(
        notes: &mut [Note],
        selected_id: Option<Uuid>,
        tags: Vec<String>,
    ) -> bool {
        let Some(id) = selected_id else {
            return false;
        };
        NoteCollection::update_tags(notes, id, tags)
    }

    pub fn request_delete(
        selected_id: &mut Option<Uuid>,
        show_delete_confirm: &mut bool,
        id: Uuid,
    ) {
        *selected_id = Some(id);
        *show_delete_confirm = true;
    }

    pub fn cancel_delete(show_delete_confirm: &mut bool) {
        *show_delete_confirm = false;
    }

    pub fn delete_confirmation_title(notes: &[Note], selected_id: Option<Uuid>) -> Option<String> {
        Self::selected_note(notes, selected_id).map(|note| note.display_title().to_string())
    }

    pub fn confirm_delete(
        notes: &mut Vec<Note>,
        selected_id: &mut Option<Uuid>,
        show_delete_confirm: &mut bool,
    ) {
        if let Some(id) = *selected_id {
            *selected_id = NoteCollection::delete_note(notes, id);
        }
        *show_delete_confirm = false;
    }

    pub fn toggle_pin(notes: &mut [Note], id: Uuid) -> bool {
        NoteCollection::toggle_pin(notes, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_empty_collection_from_missing_selection() {
        let empty_notes = Vec::new();
        assert_eq!(
            NoteWorkspace::display_state(&empty_notes, None),
            WorkspaceDisplayState::EmptyCollection
        );

        let note = Note::new("First".to_string(), String::new());
        let notes = vec![note];
        assert_eq!(
            NoteWorkspace::display_state(&notes, None),
            WorkspaceDisplayState::NoNoteSelected
        );
    }

    #[test]
    fn creates_selects_and_updates_the_selected_note() {
        let mut notes = Vec::new();
        let created = NoteWorkspace::create_note(&mut notes);
        let selected_id = created.selected_id;

        assert_eq!(
            NoteWorkspace::selected_note(&notes, selected_id)
                .unwrap()
                .id,
            notes[0].id
        );
        assert!(created.should_focus_title);

        assert!(NoteWorkspace::update_selected_title(
            &mut notes,
            selected_id,
            "Title".to_string()
        ));
        assert!(NoteWorkspace::update_selected_content(
            &mut notes,
            selected_id,
            "Content".to_string()
        ));
        assert!(NoteWorkspace::update_selected_tags(
            &mut notes,
            selected_id,
            vec!["work".to_string()]
        ));

        let selected = NoteWorkspace::selected_note(&notes, selected_id).unwrap();
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
        let mut notes = vec![first, second];
        let mut selected_id = None;
        let mut show_delete_confirm = false;

        NoteWorkspace::request_delete(&mut selected_id, &mut show_delete_confirm, first_id);
        assert_eq!(selected_id, Some(first_id));
        assert!(show_delete_confirm);

        NoteWorkspace::confirm_delete(&mut notes, &mut selected_id, &mut show_delete_confirm);
        assert_eq!(selected_id, Some(second_id));
        assert!(!show_delete_confirm);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, second_id);
    }

    #[test]
    fn delete_confirmation_identifies_the_selected_note() {
        let note = Note::new("Delete me".to_string(), String::new());
        let note_id = note.id;
        let notes = vec![note];

        assert_eq!(
            NoteWorkspace::delete_confirmation_title(&notes, Some(note_id)),
            Some("Delete me".to_string())
        );
    }

    #[test]
    fn cancelled_delete_leaves_notes_unchanged() {
        let note = Note::new("First".to_string(), String::new());
        let mut show_delete_confirm = true;
        let notes = [note];

        NoteWorkspace::cancel_delete(&mut show_delete_confirm);

        assert!(!show_delete_confirm);
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn pinning_changes_note_order_in_projection() {
        let first = Note::new("First".to_string(), String::new());
        let second = Note::new("Second".to_string(), String::new());
        let second_id = second.id;
        let mut notes = vec![first, second];

        assert!(NoteWorkspace::toggle_pin(&mut notes, second_id));
        assert!(notes[1].is_pinned);
    }
}
