#![forbid(unsafe_code)]

//! Toolkit-independent Noter domain behavior shared by migration frontends.

pub mod backup;
pub mod markdown_editing;
#[path = "../../../src/ui/markdown_preview.rs"]
pub mod markdown_preview;
pub mod transition;

#[path = "../../../src/ui/editor_view.rs"]
pub mod editor_view;
#[path = "../../../src/notes/model.rs"]
pub mod model;
#[path = "../../../src/notes/collection.rs"]
pub mod note_collection;
#[path = "../../../src/notes/discovery.rs"]
pub mod note_discovery;
#[path = "../../../src/notes/list_interaction.rs"]
pub mod note_list_interaction;
#[path = "../../../src/notes/workspace.rs"]
pub mod note_workspace;
#[path = "../../../src/ui/responsive_navigation.rs"]
pub mod responsive_navigation;
#[path = "../../../src/notes/sample.rs"]
pub mod sample_notes;
#[path = "../../../src/notes/search_query.rs"]
pub mod search_query;
#[path = "../../../src/storage/recovery.rs"]
pub mod storage_recovery;
#[path = "../../../src/notes/tag_rules.rs"]
pub mod tag_rules;

pub use model::Note;
pub use note_workspace::NoteWorkspace;
