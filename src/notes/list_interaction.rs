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

    pub fn render_model(&self, notes: &[Note], selected_id: Option<Uuid>) -> NoteListRenderModel {
        let projection = self.project_notes(notes, selected_id);
        let total_notes = notes.len();
        NoteListRenderModel {
            display_state: self.display_state(total_notes, &projection),
            result_status: self.result_status(total_notes, &projection),
            filtered_empty_message: self.filtered_empty_message(),
            projection,
        }
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

    pub fn result_status(
        &self,
        total_notes: usize,
        projection: &NoteListProjection,
    ) -> Option<NoteListResultStatus> {
        if total_notes == 0 || !projection.has_active_filter {
            return None;
        }

        let context = self.active_filter_context();
        Some(NoteListResultStatus {
            text: format!(
                "{} {} {}",
                projection.rows.len(),
                match_noun(projection.rows.len()),
                context
            ),
        })
    }

    pub fn filtered_empty_message(&self) -> NoteListFilteredEmptyMessage {
        let search = self.trimmed_committed_search();
        let tag = self.trimmed_active_tag();

        let title = match (search, tag) {
            (Some(search), Some(tag)) => format!("No notes match search: {search} in #{tag}"),
            (Some(search), None) => format!("No notes match search: {search}"),
            (None, Some(tag)) => format!("No notes tagged #{tag}"),
            (None, None) => "No notes found".to_string(),
        };

        let body = match (search, tag) {
            (Some(_), Some(_)) => "Try a different search term or clear the Tag filter.",
            (Some(_), None) => "Try a different search term.",
            (None, Some(_)) => "Clear the Tag filter to return to all Notes.",
            (None, None) => "Try a different search term.",
        };

        NoteListFilteredEmptyMessage { title, body }
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

    fn active_filter_context(&self) -> String {
        match (self.trimmed_committed_search(), self.trimmed_active_tag()) {
            (Some(search), Some(tag)) => format!("for search: {search} in #{tag}"),
            (Some(search), None) => format!("for search: {search}"),
            (None, Some(tag)) => format!("in #{tag}"),
            (None, None) => "shown".to_string(),
        }
    }

    fn trimmed_committed_search(&self) -> Option<&str> {
        let search = self.committed_search.trim();
        (!search.is_empty()).then_some(search)
    }

    fn trimmed_active_tag(&self) -> Option<&str> {
        self.active_tag
            .as_deref()
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
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
pub struct NoteListResultStatus {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteListFilteredEmptyMessage {
    pub title: String,
    pub body: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteListRenderModel {
    pub projection: NoteListProjection,
    pub display_state: NoteListDisplayState,
    pub result_status: Option<NoteListResultStatus>,
    pub filtered_empty_message: NoteListFilteredEmptyMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteActionControls {
    pub pin_label: &'static str,
    pub pin_command: NoteListCommand,
    pub delete_command: NoteListCommand,
}

fn match_noun(count: usize) -> &'static str {
    if count == 1 { "match" } else { "matches" }
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

    #[test]
    fn active_search_and_tag_filters_explain_visible_results_and_filtered_empty_state() {
        let mut launch = Note::new("Launch Plan".to_string(), "Ship the release".to_string());
        launch.tags = vec!["Work".to_string()];
        let mut groceries = Note::new("Groceries".to_string(), "Buy apples".to_string());
        groceries.tags = vec!["Personal".to_string()];
        let notes = vec![launch, groceries];

        let mut interaction = NoteListInteraction::default();
        let unfiltered = interaction.project_notes(&notes, None);
        assert_eq!(interaction.result_status(notes.len(), &unfiltered), None);

        interaction.edit_search("launch".to_string());
        interaction.commit_search();
        let search_projection = interaction.project_notes(&notes, None);
        assert_eq!(
            interaction.result_status(notes.len(), &search_projection),
            Some(NoteListResultStatus {
                text: "1 match for search: launch".to_string(),
            })
        );

        interaction.select_tag("Personal".to_string());
        let filtered_empty = interaction.project_notes(&notes, None);
        assert_eq!(
            interaction.result_status(notes.len(), &filtered_empty),
            Some(NoteListResultStatus {
                text: "0 matches for search: launch in #Personal".to_string(),
            })
        );
        assert_eq!(
            interaction.filtered_empty_message(),
            NoteListFilteredEmptyMessage {
                title: "No notes match search: launch in #Personal".to_string(),
                body: "Try a different search term or clear the Tag filter.",
            }
        );

        let empty_collection = interaction.project_notes(&[], None);
        assert_eq!(interaction.result_status(0, &empty_collection), None);
        assert_eq!(
            interaction.display_state(0, &empty_collection),
            NoteListDisplayState::EmptyCollection
        );
    }

    #[test]
    fn render_model_returns_rows_status_display_state_and_empty_copy_together() {
        let mut launch = Note::new("Launch Plan".to_string(), "Ship the release".to_string());
        launch.tags = vec!["Work".to_string()];
        let mut personal = Note::new("Personal".to_string(), "Call home".to_string());
        personal.tags = vec!["Personal".to_string()];
        let notes = vec![launch, personal];

        let mut interaction = NoteListInteraction::default();
        interaction.edit_search("launch".to_string());
        interaction.commit_search();
        interaction.select_tag("Personal".to_string());

        let model = interaction.render_model(&notes, None);

        assert!(model.projection.rows.is_empty());
        assert_eq!(model.display_state, NoteListDisplayState::FilteredEmpty);
        assert_eq!(
            model.result_status,
            Some(NoteListResultStatus {
                text: "0 matches for search: launch in #Personal".to_string(),
            })
        );
        assert_eq!(
            model.filtered_empty_message,
            NoteListFilteredEmptyMessage {
                title: "No notes match search: launch in #Personal".to_string(),
                body: "Try a different search term or clear the Tag filter.",
            }
        );
    }
}
