use crate::model::Note;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteCreation {
    pub selected_id: Option<Uuid>,
    pub should_focus_title: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoteCollection;

impl NoteCollection {
    pub fn create_note(notes: &mut Vec<Note>) -> NoteCreation {
        let note = Note::new(String::new(), String::new());
        let selected_id = Some(note.id);
        notes.insert(0, note);
        NoteCreation {
            selected_id,
            should_focus_title: true,
        }
    }

    pub fn update_title(notes: &mut [Note], id: Uuid, title: String) -> bool {
        Self::update_note(notes, id, |note| note.title = title)
    }

    pub fn update_content(notes: &mut [Note], id: Uuid, content: String) -> bool {
        Self::update_note(notes, id, |note| note.content = content)
    }

    pub fn update_tags(notes: &mut [Note], id: Uuid, tags: Vec<String>) -> bool {
        let Some(note) = notes.iter_mut().find(|note| note.id == id) else {
            return false;
        };

        if note.tags == tags {
            return false;
        }

        note.tags = tags;
        note.last_modified = Utc::now();
        true
    }

    pub fn toggle_pin(notes: &mut [Note], id: Uuid) -> bool {
        Self::update_note(notes, id, |note| note.is_pinned = !note.is_pinned)
    }

    pub fn delete_note(notes: &mut Vec<Note>, id: Uuid) -> Option<Uuid> {
        notes.retain(|note| note.id != id);
        notes.first().map(|note| note.id)
    }

    fn update_note(notes: &mut [Note], id: Uuid, update: impl FnOnce(&mut Note)) -> bool {
        let Some(note) = notes.iter_mut().find(|note| note.id == id) else {
            return false;
        };

        update(note);
        note.last_modified = Utc::now();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_a_note_inserts_selects_and_requests_title_focus() {
        let mut notes = Vec::new();
        let created = NoteCollection::create_note(&mut notes);

        assert_eq!(notes.len(), 1);
        assert_eq!(created.selected_id, Some(notes[0].id));
        assert!(created.should_focus_title);
    }

    #[test]
    fn editing_a_note_updates_fields_and_modified_time() {
        let mut note = Note::new("Old".to_string(), "Old content".to_string());
        let id = note.id;
        let original_modified = note.last_modified;
        note.last_modified = original_modified - chrono::Duration::seconds(5);
        let mut notes = vec![note];

        assert!(NoteCollection::update_title(
            &mut notes,
            id,
            "New".to_string()
        ));
        assert_eq!(notes[0].title, "New");
        assert!(notes[0].last_modified > original_modified - chrono::Duration::seconds(5));

        assert!(NoteCollection::update_content(
            &mut notes,
            id,
            "New content".to_string()
        ));
        assert_eq!(notes[0].content, "New content");
    }

    #[test]
    fn tag_updates_only_modify_when_tags_change() {
        let mut note = Note::new("Note".to_string(), "Content".to_string());
        note.tags = vec!["work".to_string()];
        let id = note.id;
        let mut notes = vec![note];

        assert!(!NoteCollection::update_tags(
            &mut notes,
            id,
            vec!["work".to_string()]
        ));
        assert!(NoteCollection::update_tags(
            &mut notes,
            id,
            vec!["rust".to_string()]
        ));
        assert_eq!(notes[0].tags, vec!["rust".to_string()]);
    }

    #[test]
    fn deleting_a_note_returns_next_selection() {
        let first = Note::new("First".to_string(), "".to_string());
        let second = Note::new("Second".to_string(), "".to_string());
        let deleted_id = first.id;
        let expected_next = second.id;
        let mut notes = vec![first, second];

        let next_selected = NoteCollection::delete_note(&mut notes, deleted_id);

        assert_eq!(next_selected, Some(expected_next));
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, expected_next);
    }
}
