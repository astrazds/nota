//! Stable dimensions and colours shared with the final web visual contract.
//!
//! Token parity (comments only — web sources of truth):
//! - Frame A light/dark: `tailwind.config.js` `apple.notebook.*`
//! - `--capture` Warm Capture Yellow: `apple.yellow` / `ThemeAccent::PrimaryFill`
//! - `--signal` notebook amber: `apple.notebook.amber` / `ThemeState::SegmentedActive`
//! - `72ch` writing plane: `src/ui/recipes.rs` `NOTE_MEASURE_CLASS` / `note_measure()`
//!
//! GTK Stylesheet rejects CSS `max-width`, so the native shell enforces the writing
//! measure in layout: Pango `ch` width × [`NativeVisualContract::editor_measure_chars`].

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePalette {
    pub frame: &'static str,
    pub surface: &'static str,
    pub sidebar: &'static str,
    pub graphite: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeVisualContract {
    pub sidebar_width: i32,
    pub footer_height: i32,
    pub editor_measure_chars: u8,
    pub note_title_font_size_px: u8,
    pub light: NativePalette,
    pub dark: NativePalette,
    pub capture: &'static str,
    pub signal: &'static str,
}

pub const NATIVE_VISUAL_CONTRACT: NativeVisualContract = NativeVisualContract {
    sidebar_width: 288,
    footer_height: 45,
    editor_measure_chars: 72,
    note_title_font_size_px: 23,
    light: NativePalette {
        frame: "#F7F5F1",
        surface: "#FDFCF9",
        sidebar: "#F0EDE6",
        graphite: "#25221F",
    },
    dark: NativePalette {
        frame: "#151311",
        surface: "#25221F",
        sidebar: "#211F1C",
        graphite: "#F7F5F1",
    },
    capture: "#FFB340",
    signal: "#E7A858",
};

pub const NATIVE_STYLESHEET: &str = include_str!("../resources/noter.css");

/// Contract writing-plane measure in CSS `ch` units (web `note_measure` parity).
pub fn writing_plane_measure_chars() -> u8 {
    NATIVE_VISUAL_CONTRACT.editor_measure_chars
}

/// Convert a measured `ch` glyph width (pixels) into the writing-plane max width.
///
/// `ch_width_px` should be the Pango pixel width of the `"0"` glyph in the body font
/// (CSS `ch` definition). Returns at least 1px.
pub fn writing_plane_max_width_px(ch_width_px: f64) -> i32 {
    let chars = f64::from(NATIVE_VISUAL_CONTRACT.editor_measure_chars);
    (ch_width_px * chars).round().max(1.0) as i32
}
