use crate::editor_view::EditorViewMode;
pub use crate::markdown_editing::MarkdownCommand;
use crate::markdown_editing::{BrowserSelection, apply_markdown_command};
use crate::markdown_preview::render_markdown_preview_body;
use crate::model::Note;

pub const HIDDEN_BY_FILTER_MESSAGE: &str = "This note is outside the current Search or Tag filter. Clear the filter in the Note List to show it there again.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritingSurfaceModel {
    pub writing: Option<WritingSurfaceEditor>,
    pub preview: Option<WritingSurfacePreview>,
    pub hidden_by_filter_message: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritingSurfaceEditor {
    pub title: String,
    pub tags: Vec<String>,
    pub content: String,
    pub formatting_tools_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritingSurfacePreview {
    pub title: String,
    pub tags: Vec<String>,
    pub body_html: String,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewSection {
    NoteTitle,
    NoteMetadata,
    MarkdownBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WritingSurfaceSelection {
    pub start_utf16: usize,
    pub end_utf16: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritingSurfaceFormattingResult {
    pub content: String,
    pub caret_utf16: usize,
}

impl WritingSurfaceModel {
    pub fn from_note(note: &Note, view_mode: EditorViewMode, is_hidden_by_filter: bool) -> Self {
        let surfaces = view_mode.surfaces();
        let hidden_by_filter_message = is_hidden_by_filter.then_some(HIDDEN_BY_FILTER_MESSAGE);
        let title = note.display_title().to_string();
        let tags = note.tags.clone();

        let writing = surfaces.writing.then(|| WritingSurfaceEditor {
            title: title.clone(),
            tags: tags.clone(),
            content: note.content.clone(),
            formatting_tools_visible: true,
        });
        let preview = surfaces.preview.then(|| WritingSurfacePreview {
            title: title.clone(),
            tags,
            body_html: render_markdown_preview_body(&title, &note.content),
        });

        Self {
            writing,
            preview,
            hidden_by_filter_message,
        }
    }
}

pub fn apply_formatting_command(
    content: &str,
    selection: WritingSurfaceSelection,
    command: MarkdownCommand,
) -> WritingSurfaceFormattingResult {
    let result = apply_markdown_command(
        content,
        BrowserSelection {
            start_utf16: selection.start_utf16,
            end_utf16: selection.end_utf16,
        },
        command,
    );

    WritingSurfaceFormattingResult {
        content: result.content,
        caret_utf16: result.caret_utf16,
    }
}

#[cfg(test)]
impl WritingSurfacePreview {
    pub fn sections(&self) -> [PreviewSection; 3] {
        [
            PreviewSection::NoteTitle,
            PreviewSection::NoteMetadata,
            PreviewSection::MarkdownBody,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_view_presents_writing_and_preview_as_one_note_context() {
        let mut note = Note::new(
            " Sprint plan ".to_string(),
            "# Sprint plan\n\nShip the editor refactor.".to_string(),
        );
        note.tags = vec!["product".to_string(), "writing".to_string()];

        let model = WritingSurfaceModel::from_note(&note, EditorViewMode::Split, true);

        let writing = model.writing.expect("split mode keeps the Writing Surface");
        assert_eq!(writing.title, "Sprint plan");
        assert_eq!(writing.tags, ["product", "writing"]);
        assert_eq!(writing.content, note.content);
        assert!(writing.formatting_tools_visible);

        let preview = model.preview.expect("split mode keeps the Preview");
        assert_eq!(
            preview.sections(),
            [
                PreviewSection::NoteTitle,
                PreviewSection::NoteMetadata,
                PreviewSection::MarkdownBody,
            ]
        );
        assert_eq!(preview.title, "Sprint plan");
        assert_eq!(preview.tags, ["product", "writing"]);
        assert!(preview.body_html.contains("Ship the editor refactor."));
        assert!(!preview.body_html.contains("<h1"));

        assert_eq!(
            model.hidden_by_filter_message,
            Some(HIDDEN_BY_FILTER_MESSAGE)
        );
    }

    #[test]
    fn formatting_commands_apply_to_writing_surface_selection() {
        let result = apply_formatting_command(
            "A😀B",
            WritingSurfaceSelection {
                start_utf16: 1,
                end_utf16: 3,
            },
            MarkdownCommand::Bold,
        );

        assert_eq!(result.content, "A**😀**B");
        assert_eq!(result.caret_utf16, 7);
    }
}
