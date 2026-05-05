use crate::model::Note;
use crate::note_discovery::{NoteListItem, NoteListProjection, project_note_list};
use uuid::Uuid;

pub const SEARCH_DEBOUNCE_MS: i32 = 200;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoteListInteraction {
    search_input: String,
    committed_search: String,
    active_tag: Option<String>,
}

impl NoteListInteraction {
    pub fn search_input(&self) -> &str {
        &self.search_input
    }

    pub fn committed_search(&self) -> &str {
        &self.committed_search
    }

    pub fn active_tag(&self) -> Option<&str> {
        self.active_tag.as_deref()
    }

    pub fn edit_search(&mut self, input: String) {
        self.search_input = input;
    }

    pub fn commit_search(&mut self) {
        self.committed_search.clone_from(&self.search_input);
    }

    pub fn select_tag(&mut self, tag: String) {
        let tag = tag.trim();
        if tag.is_empty() {
            self.active_tag = None;
        } else {
            self.active_tag = Some(tag.to_string());
        }
    }

    pub fn clear_tag(&mut self) {
        self.active_tag = None;
    }

    pub fn project_notes(&self, notes: &[Note], selected_id: Option<Uuid>) -> NoteListProjection {
        project_note_list(
            notes,
            selected_id,
            &self.committed_search,
            self.active_tag(),
        )
    }

    pub fn display_state(
        &self,
        total_notes: usize,
        projection: &NoteListProjection,
    ) -> NoteListDisplayState {
        if total_notes == 0 {
            NoteListDisplayState::EmptyCollection
        } else if projection.rows.is_empty() && projection.has_active_filter {
            NoteListDisplayState::FilteredEmpty
        } else {
            NoteListDisplayState::Rows
        }
    }

    pub fn select_row(&self, id: Uuid) -> NoteListCommand {
        NoteListCommand::SelectNote(id)
    }

    pub fn note_actions(&self, row: &NoteListItem) -> NoteActionControls {
        NoteActionControls {
            pin_label: if row.is_pinned { "Unpin" } else { "Pin" },
            pin_command: NoteListCommand::TogglePin(row.id),
            delete_command: NoteListCommand::RequestDelete(row.id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteListDisplayState {
    EmptyCollection,
    FilteredEmpty,
    Rows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteListCommand {
    SelectNote(Uuid),
    TogglePin(Uuid),
    RequestDelete(Uuid),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteActionControls {
    pub pin_label: &'static str,
    pub pin_command: NoteListCommand,
    pub delete_command: NoteListCommand,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_search_and_active_tag_distinguish_filtered_empty_from_empty_collection() {
        let mut work_note = Note::new("Sprint plan".to_string(), "Draft release notes".to_string());
        work_note.tags = vec!["Work".to_string()];
        let mut personal_note = Note::new("Groceries".to_string(), "Buy apples".to_string());
        personal_note.tags = vec!["Personal".to_string()];
        let notes = vec![work_note, personal_note];

        let mut interaction = NoteListInteraction::default();

        interaction.edit_search("draft".to_string());
        let before_commit = interaction.project_notes(&notes, None);
        assert_eq!(before_commit.rows.len(), 2);

        interaction.commit_search();
        let after_commit = interaction.project_notes(&notes, None);
        assert_eq!(
            after_commit
                .rows
                .iter()
                .map(|row| row.display_title.as_str())
                .collect::<Vec<_>>(),
            vec!["Sprint plan"]
        );

        interaction.select_tag("Personal".to_string());
        let filtered_empty = interaction.project_notes(&notes, None);
        assert_eq!(filtered_empty.rows.len(), 0);
        assert_eq!(
            interaction.display_state(notes.len(), &filtered_empty),
            NoteListDisplayState::FilteredEmpty
        );

        interaction.clear_tag();
        let tag_cleared = interaction.project_notes(&notes, None);
        assert_eq!(tag_cleared.rows.len(), 1);

        let empty_collection = interaction.project_notes(&[], None);
        assert_eq!(
            interaction.display_state(0, &empty_collection),
            NoteListDisplayState::EmptyCollection
        );
    }

    #[test]
    fn row_selection_and_note_actions_expose_stable_commands() {
        let mut note = Note::new("Pinned".to_string(), String::new());
        note.is_pinned = true;
        let row = NoteListInteraction::default()
            .project_notes(&[note.clone()], Some(note.id))
            .rows
            .remove(0);

        let interaction = NoteListInteraction::default();
        assert_eq!(
            interaction.select_row(row.id),
            NoteListCommand::SelectNote(note.id)
        );

        let actions = interaction.note_actions(&row);
        assert_eq!(actions.pin_label, "Unpin");
        assert_eq!(actions.pin_command, NoteListCommand::TogglePin(note.id));
        assert_eq!(
            actions.delete_command,
            NoteListCommand::RequestDelete(note.id)
        );
    }
}
