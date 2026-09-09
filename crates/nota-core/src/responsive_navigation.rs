use crate::editor_view::EditorViewMode;

pub const WIDE_VIEWPORT_MIN_WIDTH: f64 = 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportClass {
    Compact,
    Wide,
}

impl ViewportClass {
    pub fn from_width(width: f64) -> Self {
        if width >= WIDE_VIEWPORT_MIN_WIDTH {
            Self::Wide
        } else {
            Self::Compact
        }
    }

    fn supports_split_view(self) -> bool {
        matches!(self, Self::Wide)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredNoteListState {
    Open,
    Closed,
}

impl StoredNoteListState {
    pub fn from_is_open(is_open: bool) -> Self {
        if is_open { Self::Open } else { Self::Closed }
    }

    pub fn is_open(self) -> bool {
        matches!(self, Self::Open)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteListPersistence {
    Persist(StoredNoteListState),
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponsiveNavigation {
    viewport: ViewportClass,
    is_note_list_visible: bool,
}

impl ResponsiveNavigation {
    pub fn initial(viewport: ViewportClass, stored: StoredNoteListState) -> Self {
        Self {
            viewport,
            is_note_list_visible: matches!(viewport, ViewportClass::Wide)
                || matches!(stored, StoredNoteListState::Open),
        }
    }

    pub fn current(viewport: ViewportClass, is_note_list_visible: bool) -> Self {
        Self {
            viewport,
            is_note_list_visible,
        }
    }

    pub fn is_note_list_visible(&self) -> bool {
        self.is_note_list_visible
    }

    pub fn toggle_note_list(&mut self) {
        self.is_note_list_visible = !self.is_note_list_visible;
    }

    pub fn note_selected(&mut self) {
        if matches!(self.viewport, ViewportClass::Compact) {
            self.is_note_list_visible = false;
        }
    }

    pub fn reclassify_viewport(&mut self, viewport: ViewportClass) {
        self.viewport = viewport;
        if matches!(viewport, ViewportClass::Wide) {
            self.is_note_list_visible = true;
        }
    }

    pub fn persistence(&self) -> NoteListPersistence {
        if matches!(self.viewport, ViewportClass::Compact) {
            NoteListPersistence::Persist(StoredNoteListState::from_is_open(
                self.is_note_list_visible,
            ))
        } else {
            NoteListPersistence::Skip
        }
    }
}

pub fn is_view_mode_available(viewport: ViewportClass, view_mode: EditorViewMode) -> bool {
    view_mode != EditorViewMode::Split || viewport.supports_split_view()
}

pub fn normalize_view_mode(viewport: ViewportClass, view_mode: EditorViewMode) -> EditorViewMode {
    if is_view_mode_available(viewport, view_mode) {
        view_mode
    } else {
        EditorViewMode::Write
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_class_uses_the_navigation_breakpoint() {
        assert_eq!(ViewportClass::from_width(375.0), ViewportClass::Compact);
        assert_eq!(
            ViewportClass::from_width(WIDE_VIEWPORT_MIN_WIDTH - 1.0),
            ViewportClass::Compact
        );
        assert_eq!(
            ViewportClass::from_width(WIDE_VIEWPORT_MIN_WIDTH),
            ViewportClass::Wide
        );
    }

    #[test]
    fn wide_viewports_show_the_note_list_by_default_even_when_storage_was_closed() {
        let navigation =
            ResponsiveNavigation::initial(ViewportClass::Wide, StoredNoteListState::Closed);

        assert!(navigation.is_note_list_visible());
    }

    #[test]
    fn compact_viewports_restore_the_stored_note_list_state() {
        assert!(
            ResponsiveNavigation::initial(ViewportClass::Compact, StoredNoteListState::Open)
                .is_note_list_visible()
        );
        assert!(
            !ResponsiveNavigation::initial(ViewportClass::Compact, StoredNoteListState::Closed)
                .is_note_list_visible()
        );
    }

    #[test]
    fn selecting_a_note_on_compact_viewports_returns_to_the_writing_surface() {
        let mut navigation =
            ResponsiveNavigation::initial(ViewportClass::Compact, StoredNoteListState::Open);

        navigation.note_selected();

        assert!(!navigation.is_note_list_visible());
    }

    #[test]
    fn selecting_a_note_on_wide_viewports_keeps_the_note_list_available() {
        let mut navigation =
            ResponsiveNavigation::initial(ViewportClass::Wide, StoredNoteListState::Closed);

        navigation.note_selected();

        assert!(navigation.is_note_list_visible());
    }

    #[test]
    fn top_bar_navigation_toggles_the_note_list() {
        let mut navigation =
            ResponsiveNavigation::initial(ViewportClass::Compact, StoredNoteListState::Closed);

        navigation.toggle_note_list();
        assert!(navigation.is_note_list_visible());

        navigation.toggle_note_list();
        assert!(!navigation.is_note_list_visible());
    }

    #[test]
    fn reclassifying_to_wide_restores_the_desktop_note_list_default() {
        let mut navigation =
            ResponsiveNavigation::initial(ViewportClass::Compact, StoredNoteListState::Closed);

        navigation.reclassify_viewport(ViewportClass::Wide);

        assert!(navigation.is_note_list_visible());
        assert_eq!(navigation.persistence(), NoteListPersistence::Skip);
    }

    #[test]
    fn split_view_mode_is_only_available_on_wide_viewports() {
        assert!(!is_view_mode_available(
            ViewportClass::Compact,
            EditorViewMode::Split
        ));
        assert_eq!(
            normalize_view_mode(ViewportClass::Compact, EditorViewMode::Split),
            EditorViewMode::Write
        );
        assert!(is_view_mode_available(
            ViewportClass::Wide,
            EditorViewMode::Split
        ));
    }

    #[test]
    fn wide_viewport_defaults_do_not_overwrite_compact_note_list_persistence() {
        let navigation =
            ResponsiveNavigation::initial(ViewportClass::Wide, StoredNoteListState::Closed);

        assert!(navigation.is_note_list_visible());
        assert_eq!(navigation.persistence(), NoteListPersistence::Skip);

        let compact_navigation =
            ResponsiveNavigation::initial(ViewportClass::Compact, StoredNoteListState::Open);
        assert_eq!(
            compact_navigation.persistence(),
            NoteListPersistence::Persist(StoredNoteListState::Open)
        );
    }
}
