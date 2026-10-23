#![forbid(unsafe_code)]

pub mod backup;
pub mod exchange;
pub mod markdown_editing;
pub mod markdown_preview;
pub mod transition;

pub mod editor_view;
pub mod identity;
pub mod layout;
pub mod model;
pub mod note_collection;
pub mod note_discovery;
pub mod note_list_interaction;
pub mod note_workspace;
pub mod product;
pub mod search_query;
pub mod tag_rules;

pub use model::Note;
pub use note_workspace::NoteWorkspace;
