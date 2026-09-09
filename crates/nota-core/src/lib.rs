#![forbid(unsafe_code)]

//! Toolkit-independent Nota domain behavior shared by migration frontends.

pub mod backup;
pub mod markdown_editing;
pub mod markdown_preview;
pub mod transition;

pub mod editor_view;
pub mod model;
pub mod note_collection;
pub mod note_discovery;
pub mod note_list_interaction;
pub mod note_workspace;
pub mod responsive_navigation;
pub mod sample_notes;
pub mod search_query;
pub mod storage_recovery;
pub mod tag_rules;

pub use model::Note;
pub use note_workspace::NoteWorkspace;
