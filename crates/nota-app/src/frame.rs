use std::path::PathBuf;

use nota_core::Note;
use nota_core::layout::LayoutFrame;
use nota_core::product::Prompt;
use nota_core::transition::ThemePreference;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameRow {
    pub id: Uuid,
    pub title: String,
    pub preview: String,
    pub date: String,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletedRow {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrameNotification {
    pub message: String,
    pub tone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrameRecovery {
    pub reason: String,
    pub can_restore_previous: bool,
}

/// What a shell paints. Shells do not choose stacking, list copy, health copy, or confirmation copy.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    pub revision: u64,
    pub edit_sequence: u64,
    pub selected_note: Option<Note>,
    pub rows: Vec<FrameRow>,
    pub recently_deleted: Vec<DeletedRow>,
    pub search_input: String,
    pub active_tag: Option<String>,
    pub tags: Vec<String>,
    pub pinned_only: bool,
    pub view_mode: String,
    pub theme: ThemePreference,
    pub save_status: String,
    pub backup_health: String,
    pub backup_detail: String,
    pub notification: Option<FrameNotification>,
    pub recovery: Option<FrameRecovery>,
    pub confirm: Option<Prompt>,
    pub suggested_export_name: String,
    pub data_directory: PathBuf,
    pub layout: LayoutFrame,
    pub list_title: String,
    pub list_body: String,
    pub drawer_open: bool,
    pub editing_tags: bool,
    pub tag_cleanup: Option<Prompt>,
    pub quarantine: String,
    pub focus_title: bool,
}
