#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorViewMode {
    Write,
    Preview,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportClass {
    Compact,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleEditorSurfaces {
    pub writing: bool,
    pub preview: bool,
}

impl EditorViewMode {
    pub fn normalized_for(self, viewport: ViewportClass) -> Self {
        match (self, viewport) {
            (Self::Split, ViewportClass::Compact) => Self::Write,
            _ => self,
        }
    }

    pub fn surfaces(self) -> VisibleEditorSurfaces {
        match self {
            Self::Write => VisibleEditorSurfaces {
                writing: true,
                preview: false,
            },
            Self::Preview => VisibleEditorSurfaces {
                writing: false,
                preview: true,
            },
            Self::Split => VisibleEditorSurfaces {
                writing: true,
                preview: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_view_is_only_available_on_wide_viewports() {
        assert_eq!(
            EditorViewMode::Split.normalized_for(ViewportClass::Compact),
            EditorViewMode::Write
        );
        assert_eq!(
            EditorViewMode::Split.normalized_for(ViewportClass::Wide),
            EditorViewMode::Split
        );
    }

    #[test]
    fn view_modes_explain_visible_surfaces() {
        assert_eq!(
            EditorViewMode::Write.surfaces(),
            VisibleEditorSurfaces {
                writing: true,
                preview: false
            }
        );
        assert_eq!(
            EditorViewMode::Preview.surfaces(),
            VisibleEditorSurfaces {
                writing: false,
                preview: true
            }
        );
        assert_eq!(
            EditorViewMode::Split.surfaces(),
            VisibleEditorSurfaces {
                writing: true,
                preview: true
            }
        );
    }
}
