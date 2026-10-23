use serde::Serialize;

use crate::editor_view::EditorViewMode;

/// Horizontal is a side by side editor. Vertical stacks the preview under the writing surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Insets {
    pub start: f64,
    pub top: f64,
    pub end: f64,
    pub bottom: f64,
}

/// The only width decision for stacking and the narrow split.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LayoutFrame {
    pub axis: Axis,
    pub drawer_fills_width: bool,
    pub footer_stacks: bool,
    pub show_writing: bool,
    pub show_preview: bool,
    pub title_px: f64,
    pub body_px: f64,
    pub writing_insets: Insets,
    pub preview_insets: Insets,
    pub split_gap_px: f64,
    pub editor_min_height_px: f64,
    pub preview_min_height_px: f64,
}

impl LayoutFrame {
    /// 560 and below stacks. Above 560 and at or below 760 is the narrow split.
    /// Editor min height is 340. Stacked preview min height is 500.
    pub fn from_width(width_px: f64, mode: EditorViewMode) -> Option<Self> {
        if width_px <= 0.0 {
            return None;
        }
        let stacked = width_px <= 560.0;
        let narrow = width_px <= 760.0;
        let surfaces = mode.surfaces();
        let split = mode == EditorViewMode::Split;
        let vertical = split && stacked;
        let title_px = if stacked {
            31.0
        } else if narrow {
            36.0
        } else {
            39.0
        };
        let body_px = if split {
            13.0
        } else if stacked {
            16.0
        } else {
            18.0
        };
        let writing_insets = if stacked {
            Insets {
                start: 24.0,
                top: 28.0,
                end: 24.0,
                bottom: 20.0,
            }
        } else if split && narrow {
            Insets {
                start: 19.0,
                top: 31.0,
                end: 19.0,
                bottom: 24.0,
            }
        } else if split {
            Insets {
                start: 27.0,
                top: 31.0,
                end: 27.0,
                bottom: 24.0,
            }
        } else {
            Insets {
                start: 42.0,
                top: 31.0,
                end: 42.0,
                bottom: 24.0,
            }
        };
        let preview_insets = if vertical {
            Insets {
                start: 0.0,
                top: 24.0,
                end: 0.0,
                bottom: 0.0,
            }
        } else if split && narrow {
            Insets {
                start: 18.0,
                top: 0.0,
                end: 0.0,
                bottom: 0.0,
            }
        } else if split {
            Insets {
                start: 25.0,
                top: 0.0,
                end: 0.0,
                bottom: 0.0,
            }
        } else {
            Insets {
                start: 0.0,
                top: 0.0,
                end: 0.0,
                bottom: 0.0,
            }
        };
        Some(Self {
            axis: if vertical {
                Axis::Vertical
            } else {
                Axis::Horizontal
            },
            drawer_fills_width: stacked,
            footer_stacks: stacked,
            show_writing: surfaces.writing,
            show_preview: surfaces.preview,
            title_px,
            body_px,
            writing_insets,
            preview_insets,
            split_gap_px: if narrow { 18.0 } else { 28.0 },
            editor_min_height_px: if vertical { 340.0 } else { 0.0 },
            preview_min_height_px: if vertical { 500.0 } else { 0.0 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_layout_stacks_at_560_and_splits_narrow_at_760() {
        let stacked = LayoutFrame::from_width(560.0, EditorViewMode::Split).unwrap();
        assert_eq!(stacked.axis, Axis::Vertical);
        assert!(stacked.drawer_fills_width);
        assert!(stacked.footer_stacks);
        assert!(stacked.show_writing);
        assert!(stacked.show_preview);
        assert_eq!(stacked.title_px, 31.0);
        assert_eq!(stacked.body_px, 13.0);
        assert_eq!(stacked.editor_min_height_px, 340.0);
        assert_eq!(stacked.preview_min_height_px, 500.0);
        assert_eq!(stacked.split_gap_px, 18.0);

        let just_above = LayoutFrame::from_width(560.1, EditorViewMode::Split).unwrap();
        assert_eq!(just_above.axis, Axis::Horizontal);
        assert!(!just_above.drawer_fills_width);
        assert_eq!(just_above.preview_min_height_px, 0.0);
        assert_eq!(just_above.title_px, 36.0);

        let narrow = LayoutFrame::from_width(760.0, EditorViewMode::Split).unwrap();
        assert_eq!(narrow.axis, Axis::Horizontal);
        assert_eq!(narrow.title_px, 36.0);
        assert_eq!(narrow.split_gap_px, 18.0);
        assert_eq!(narrow.body_px, 13.0);

        let wide = LayoutFrame::from_width(760.1, EditorViewMode::Split).unwrap();
        assert_eq!(wide.title_px, 39.0);
        assert_eq!(wide.body_px, 13.0);
        assert_eq!(wide.split_gap_px, 28.0);

        let reading = LayoutFrame::from_width(1200.0, EditorViewMode::Write).unwrap();
        assert!(reading.show_writing);
        assert!(!reading.show_preview);
        assert_eq!(reading.body_px, 18.0);
        assert_eq!(reading.editor_min_height_px, 0.0);

        assert!(LayoutFrame::from_width(0.0, EditorViewMode::Write).is_none());
        assert!(LayoutFrame::from_width(-1.0, EditorViewMode::Split).is_none());
    }
}
