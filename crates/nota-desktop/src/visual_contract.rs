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
    pub editor_measure_px: i32,
    pub note_title_font_size_px: u8,
    pub light: NativePalette,
    pub dark: NativePalette,
    pub capture: &'static str,
    pub signal: &'static str,
}

pub const NATIVE_VISUAL_CONTRACT: NativeVisualContract = NativeVisualContract {
    sidebar_width: 278,
    footer_height: 65,
    editor_measure_px: 600,
    note_title_font_size_px: 39,
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

pub const NATIVE_STYLESHEET: &str = include_str!("../resources/nota.css");
