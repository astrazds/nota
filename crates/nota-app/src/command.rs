use std::path::PathBuf;

use nota_core::editor_view::EditorViewMode;
use nota_core::markdown_editing::MarkdownCommand;
use nota_core::transition::ThemePreference;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::frame::Frame;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextSelection {
    Utf8 { start: usize, end: usize },
    Utf16 { start: usize, end: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatEdit {
    pub content: String,
    pub caret_utf16: usize,
    pub caret_byte: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Frame(Box<Frame>),
    Edit(FormatEdit),
    Preview(String),
    External(Option<String>),
}

#[derive(Debug)]
pub struct DecodeError {
    pub message: String,
}

impl DecodeError {
    fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

macro_rules! field_ty {
    (OptionString) => {
        Option<String>
    };
    ($ty:ident) => {
        $ty
    };
}

macro_rules! pull {
    ($map:ident, $field:ident, f64) => {
        pull_f64($map, stringify!($field))?
    };
    ($map:ident, $field:ident, String) => {
        pull_string($map, stringify!($field))?
    };
    ($map:ident, $field:ident, bool) => {
        pull_bool($map, stringify!($field))?
    };
    ($map:ident, $field:ident, u64) => {
        pull_u64($map, stringify!($field))?
    };
    ($map:ident, $field:ident, Uuid) => {
        pull_uuid($map, stringify!($field))?
    };
    ($map:ident, $field:ident, PathBuf) => {
        pull_path($map, stringify!($field))?
    };
    ($map:ident, $field:ident, EditorViewMode) => {
        pull_view_mode($map, stringify!($field))?
    };
    ($map:ident, $field:ident, ThemePreference) => {
        pull_theme($map, stringify!($field))?
    };
    ($map:ident, $field:ident, MarkdownCommand) => {
        pull_markdown($map, stringify!($field))?
    };
    ($map:ident, $field:ident, OptionString) => {
        pull_option_string($map, stringify!($field))?
    };
}

macro_rules! commands {
    (@body $map:ident $wire:literal Format { $($rest:tt)* } -> Edit) => {
        {
            let _ = $wire;
            deny_unknown(
                $map,
                &["command", "content", "start_utf16", "end_utf16", "kind"],
            )?;
            let content = pull_string($map, "content")?;
            let start = pull_usize($map, "start_utf16")?;
            let end = pull_usize($map, "end_utf16")?;
            let kind = pull_markdown($map, "kind")?;
            Ok(Command::Format {
                content,
                selection: TextSelection::Utf16 { start, end },
                kind,
            })
        }
    };
    (@body $map:ident $wire:literal $name:ident { $($field:ident : $ty:ident),+ } -> $effect:ident) => {
        {
            deny_unknown($map, &["command", $(stringify!($field)),*])?;
            Ok(Command::$name { $($field: pull!($map, $field, $ty)),* })
        }
    };
    (@body $map:ident $wire:literal $name:ident { } -> $effect:ident) => {
        {
            deny_unknown($map, &["command"])?;
            Ok(Command::$name {})
        }
    };
    ($( $wire:literal $name:ident { $($field:ident : $ty:ident),* $(,)? } -> $effect:ident ),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Command {
            $($name { $($field: field_ty!($ty)),* },)*
        }

        pub fn decode_command(value: &Value) -> Result<Command, DecodeError> {
            let map = value
                .as_object()
                .ok_or_else(|| DecodeError::new("command must be a JSON object"))?;
            let name = map
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| DecodeError::new("command is required"))?;
            match name {
                $($wire => commands!(@body map $wire $name { $($field : $ty),* } -> $effect),)*
                other => Err(DecodeError::new(format!("unknown command {other}"))),
            }
        }
    };
}

commands! {
    "snapshot" Snapshot {} -> Frame,
    "new_note" NewNote {} -> Frame,
    "flush" Flush {} -> Frame,
    "resize" Resize { width_px: f64 } -> Frame,
    "set_view_mode" SetViewMode { mode: EditorViewMode } -> Frame,
    "set_theme" SetTheme { theme: ThemePreference } -> Frame,
    "toggle_theme" ToggleTheme {} -> Frame,
    "toggle_navigation" ToggleNavigation {} -> Frame,
    "select_note" SelectNote { id: Uuid } -> Frame,
    "edit_note" EditNote { id: Uuid, edit_sequence: u64, title: String, content: String, tags_input: String } -> Frame,
    "search" Search { query: String } -> Frame,
    "filter_tag" FilterTag { tag: OptionString } -> Frame,
    "filter_pinned" FilterPinned { pinned: bool } -> Frame,
    "toggle_pin" TogglePin { id: Uuid } -> Frame,
    "ask_delete" AskDelete { id: Uuid } -> Frame,
    "ask_permanent_delete" AskPermanentDelete { id: Uuid } -> Frame,
    "ask_clear_deleted" AskClearDeleted {} -> Frame,
    "accept_confirm" AcceptConfirm {} -> Frame,
    "dismiss_confirm" DismissConfirm {} -> Frame,
    "restore_note" RestoreNote { id: Uuid } -> Frame,
    "import_notebook" ImportNotebook { json: String } -> Frame,
    "export_notebook" ExportNotebook { path: PathBuf } -> Frame,
    "restore_previous" RestorePrevious {} -> Frame,
    "start_empty" StartEmpty {} -> Frame,
    "format" Format { content: String, selection: TextSelection, kind: MarkdownCommand } -> Edit,
    "preview" Preview { title: String, content: String } -> Preview,
    "external_navigation" ExternalNavigation { uri: String, user_activated: bool } -> External,
    "apply_tag_cleanup" ApplyTagCleanup {} -> Frame,
    "accept_tag_suggestion" AcceptTagSuggestion { input: String } -> Frame,
    "start_edit_tags" StartEditTags {} -> Frame,
    "finish_edit_tags" FinishEditTags {} -> Frame,
    "dismiss_notification" DismissNotification { generation: u64 } -> Frame,
}

fn deny_unknown(map: &Map<String, Value>, allowed: &[&str]) -> Result<(), DecodeError> {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(DecodeError::new(format!("unknown field {key}")));
        }
    }
    Ok(())
}

fn required<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Value, DecodeError> {
    map.get(key)
        .ok_or_else(|| DecodeError::new(format!("missing field {key}")))
}

fn pull_f64(map: &Map<String, Value>, key: &str) -> Result<f64, DecodeError> {
    required(map, key)?
        .as_f64()
        .ok_or_else(|| DecodeError::new(format!("{key} must be a number")))
}

fn pull_string(map: &Map<String, Value>, key: &str) -> Result<String, DecodeError> {
    required(map, key)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| DecodeError::new(format!("{key} must be a string")))
}

fn pull_bool(map: &Map<String, Value>, key: &str) -> Result<bool, DecodeError> {
    required(map, key)?
        .as_bool()
        .ok_or_else(|| DecodeError::new(format!("{key} must be a boolean")))
}

fn pull_u64(map: &Map<String, Value>, key: &str) -> Result<u64, DecodeError> {
    required(map, key)?
        .as_u64()
        .ok_or_else(|| DecodeError::new(format!("{key} must be an integer")))
}

fn pull_usize(map: &Map<String, Value>, key: &str) -> Result<usize, DecodeError> {
    let value = pull_u64(map, key)?;
    usize::try_from(value).map_err(|_| DecodeError::new(format!("{key} is too large")))
}

fn pull_uuid(map: &Map<String, Value>, key: &str) -> Result<Uuid, DecodeError> {
    let text = pull_string(map, key)?;
    Uuid::parse_str(&text).map_err(|_| DecodeError::new(format!("{key} must be a uuid")))
}

fn pull_path(map: &Map<String, Value>, key: &str) -> Result<PathBuf, DecodeError> {
    Ok(PathBuf::from(pull_string(map, key)?))
}

fn pull_option_string(map: &Map<String, Value>, key: &str) -> Result<Option<String>, DecodeError> {
    let value = required(map, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_str()
            .map(|text| Some(text.to_string()))
            .ok_or_else(|| DecodeError::new(format!("{key} must be a string or null")))
    }
}

fn pull_view_mode(map: &Map<String, Value>, key: &str) -> Result<EditorViewMode, DecodeError> {
    match pull_string(map, key)?.as_str() {
        "write" => Ok(EditorViewMode::Write),
        "preview" => Ok(EditorViewMode::Preview),
        "split" => Ok(EditorViewMode::Split),
        _ => Err(DecodeError::new(format!("{key} is not a view mode"))),
    }
}

fn pull_theme(map: &Map<String, Value>, key: &str) -> Result<ThemePreference, DecodeError> {
    match pull_string(map, key)?.as_str() {
        "light" => Ok(ThemePreference::Light),
        "dark" => Ok(ThemePreference::Dark),
        "system" => Ok(ThemePreference::System),
        _ => Err(DecodeError::new(format!("{key} is not a theme"))),
    }
}

fn pull_markdown(map: &Map<String, Value>, key: &str) -> Result<MarkdownCommand, DecodeError> {
    match pull_string(map, key)?.as_str() {
        "bold" => Ok(MarkdownCommand::Bold),
        "italic" => Ok(MarkdownCommand::Italic),
        "heading" => Ok(MarkdownCommand::Heading),
        "bullet_list" => Ok(MarkdownCommand::BulletList),
        "link" => Ok(MarkdownCommand::Link),
        "strikethrough" => Ok(MarkdownCommand::Strikethrough),
        "task_list" => Ok(MarkdownCommand::TaskList),
        "table" => Ok(MarkdownCommand::Table),
        _ => Err(DecodeError::new(format!("{key} is not a format"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_wire_keeps_utf16_offsets() {
        let command = decode_command(&json!({
            "command": "format",
            "content": "ab",
            "start_utf16": 0,
            "end_utf16": 2,
            "kind": "bold"
        }))
        .unwrap();
        assert!(matches!(
            command,
            Command::Format {
                selection: TextSelection::Utf16 { start: 0, end: 2 },
                kind: MarkdownCommand::Bold,
                ..
            }
        ));
    }

    #[test]
    fn preview_rejects_a_shell_chosen_layout() {
        let error = decode_command(&json!({
            "command": "preview",
            "title": "Notebook",
            "content": "Body",
            "layout": "editing"
        }))
        .unwrap_err();
        assert!(error.message.contains("layout"));
    }
}
