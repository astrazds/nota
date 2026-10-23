#![forbid(unsafe_code)]

pub mod app;
pub mod command;
pub mod frame;
pub mod persistence;
pub mod preview;
pub mod session;
pub mod storage;

pub use frame::Frame;
pub use nota_core::Note;
pub use nota_core::backup::{BackupHealthRecord, export_flat_collection_backup};
pub use nota_core::editor_view::EditorViewMode;
pub use nota_core::exchange::{ImportDestination, stage_import};
pub use nota_core::layout::{Axis, LayoutFrame};
pub use nota_core::markdown_editing::{
    ByteSelection, MARKDOWN_CHEATSHEET_SECTIONS, MarkdownCommand,
};
pub use nota_core::note_discovery::HighlightSegment;
pub use nota_core::note_list_interaction::{NoteListDisplayState, SEARCH_DEBOUNCE_MS};
pub use nota_core::note_workspace::FocusIntent;
pub use nota_core::product::{ProductText, Prompt};
pub use nota_core::transition::{ThemePreference, TransitionError, export_desktop_transition};
pub use session::{OpenOptionsJson, Session, SessionError};

pub const APPLICATION_ID: &str = "net.astrazds.Nota";
