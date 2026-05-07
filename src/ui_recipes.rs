use crate::theme::{ThemeState, ThemeSurface, ThemeText};

pub fn search_hint() -> String {
    format!(
        "mt-1.5 rounded-md border p-2 text-[11px] shadow-sm {} {}",
        ThemeSurface::EditorChrome.classes(),
        ThemeText::Primary.classes()
    )
}

pub fn sidebar_footer() -> String {
    format!(
        "h-[45px] gap-x-2 gap-y-1 px-3 py-1.5 border-t border-apple-gray-300 dark:border-apple-dark-border flex flex-wrap items-center text-[11px] leading-4 {}",
        ThemeText::Subtle.classes()
    )
}

pub fn editor_footer() -> String {
    format!(
        "h-[45px] shrink-0 gap-x-2 gap-y-1 border-t border-apple-gray-300 px-3 py-1.5 text-[11px] leading-4 dark:border-apple-dark-border flex flex-wrap items-center justify-center {}",
        ThemeSurface::EditorChrome.classes()
    )
}

pub fn compact_controls() -> &'static str {
    "flex min-w-0 flex-wrap items-center justify-center gap-x-2 gap-y-1"
}

pub fn compact_segmented_button(is_active: bool, is_desktop_only: bool) -> String {
    let visibility = if is_desktop_only {
        "hidden lg:inline-flex"
    } else {
        "inline-flex"
    };
    let state_classes = if is_active {
        ThemeState::SegmentedActive.classes()
    } else {
        ThemeState::SegmentedIdle.classes()
    };

    format!(
        "{visibility} h-9 min-w-[2.25rem] items-center justify-center rounded-md px-3 text-xs transition-colors md:h-auto md:min-w-0 md:px-1.5 md:py-0.5 md:text-[11px] {state_classes}"
    )
}

pub fn compact_help_button() -> String {
    format!(
        "inline-flex h-9 w-9 items-center justify-center rounded-md px-0 text-xs transition-colors md:h-auto md:w-auto md:px-1.5 md:py-0.5 md:text-[11px] {}",
        ThemeState::SegmentedIdle.classes()
    )
}

pub fn backup_footer_button() -> String {
    format!(
        "cursor-pointer rounded-md px-1.5 py-0.5 text-[11px] {}",
        ThemeState::SegmentedIdle.classes()
    )
}

pub fn danger_footer_button() -> String {
    format!(
        "ml-auto cursor-pointer rounded-md px-1.5 py-0.5 text-[11px] {}",
        ThemeState::DangerMenuItem.classes()
    )
}

pub fn tag_pill() -> String {
    format!(
        "rounded-full px-2 py-0.5 text-xs {}",
        ThemeState::TagPill.classes()
    )
}

pub fn note_row(is_selected: bool) -> String {
    let state_classes = if is_selected {
        ThemeState::NoteRowSelected.classes()
    } else {
        ThemeState::NoteRowIdle.classes()
    };

    format!(
        "px-4 py-3 border-b cursor-pointer transition-colors duration-200 ease-out group {state_classes}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_footer_recipes_keep_sidebar_and_editor_rhythm_consistent() {
        let sidebar = sidebar_footer();
        let editor = editor_footer();

        for classes in [&sidebar, &editor] {
            assert!(classes.contains("h-[45px]"));
            assert!(classes.contains("px-3"));
            assert!(classes.contains("py-1.5"));
            assert!(classes.contains("text-[11px]"));
            assert!(classes.contains("leading-4"));
            assert!(classes.contains("border-t"));
            assert!(classes.contains("border-apple-gray-300"));
            assert!(classes.contains("dark:border-apple-dark-border"));
            assert!(classes.contains("flex"));
            assert!(classes.contains("flex-wrap"));
            assert!(classes.contains("items-center"));
        }

        assert!(editor.contains("justify-center"));
    }

    #[test]
    fn compact_control_recipes_keep_button_sizing_and_visibility_stable() {
        let controls = compact_controls();
        let idle = compact_segmented_button(false, false);
        let active = compact_segmented_button(true, false);
        let split = compact_segmented_button(false, true);
        let help = compact_help_button();
        let backup = backup_footer_button();
        let danger = danger_footer_button();

        assert!(controls.contains("gap-x-2"));
        assert!(controls.contains("gap-y-1"));

        for classes in [&idle, &active, &help] {
            assert!(classes.contains("rounded-md"));
            assert!(classes.contains("h-9"));
            assert!(classes.contains("md:px-1.5"));
            assert!(classes.contains("md:py-0.5"));
            assert!(classes.contains("md:text-[11px]"));
        }

        assert!(idle.contains("min-w-[2.25rem]"));
        assert!(active.contains("min-w-[2.25rem]"));
        assert!(help.contains("w-9"));

        for classes in [&backup, &danger] {
            assert!(classes.contains("rounded-md"));
            assert!(classes.contains("px-1.5"));
            assert!(classes.contains("py-0.5"));
            assert!(classes.contains("text-[11px]"));
        }

        assert!(split.starts_with("hidden lg:inline-flex"));
        assert!(active.contains("border-apple-yellow"));
        assert!(danger.contains("text-red"));
    }

    #[test]
    fn search_hint_tag_and_selected_note_recipes_are_theme_aware() {
        let search = search_hint();
        let tag = tag_pill();
        let selected = note_row(true);
        let idle = note_row(false);

        assert!(!search.contains("absolute"));
        assert!(search.contains("mt-1.5"));
        assert!(search.contains("p-2"));
        assert!(search.contains("shadow-sm"));
        assert!(search.contains("text-gray-900"));
        assert!(search.contains("dark:text-white"));

        assert!(tag.contains("rounded-full"));
        assert!(tag.contains("text-xs"));
        assert!(tag.contains("dark:bg-white/10"));

        assert!(selected.contains("border-apple-yellow"));
        assert!(selected.contains("ring-1"));
        assert!(selected.contains("dark:bg-apple-yellow/20"));
        assert!(idle.contains("hover:bg-apple-gray-200"));
    }
}
