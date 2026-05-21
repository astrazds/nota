use crate::theme::{ThemeState, ThemeSurface, ThemeText};

const COMPACT_FOOTER_HEIGHT_CLASS: &str = "h-[45px]";
const NOTE_MEASURE_CLASS: &str = "w-full max-w-[72ch]";
const PANE_INLINE_INSET_CLASS: &str = "px-6 md:px-8";
const PANE_TOP_INSET_CLASS: &str = "pt-4 md:pt-5";

pub fn search_hint() -> String {
    format!(
        "pointer-events-none absolute left-0 right-0 top-full z-30 mt-1.5 rounded-md border p-2 text-[11px] shadow-sm {} {}",
        ThemeSurface::EditorChrome.classes(),
        ThemeText::Primary.classes()
    )
}

pub fn search_input() -> String {
    "w-full rounded-lg bg-apple-notebook-border/70 py-1.5 pl-10 pr-10 text-sm text-apple-notebook-graphite placeholder:text-gray-400 transition-colors focus:bg-apple-notebook-border focus:outline-none dark:bg-white/10 dark:text-gray-100 dark:placeholder:text-gray-500 dark:focus:bg-white/15"
        .to_string()
}

pub fn search_clear_button() -> String {
    "absolute inset-y-0 right-1 my-auto inline-flex h-7 w-7 items-center justify-center rounded-md text-gray-400 transition-colors hover:bg-apple-notebook-borderStrong hover:text-apple-notebook-graphite focus:outline-none focus:ring-2 focus:ring-apple-notebook-amber focus:ring-offset-1 dark:text-gray-500 dark:hover:bg-white/10 dark:hover:text-gray-200 dark:focus:ring-offset-apple-notebook-darkSidebar"
        .to_string()
}

pub fn search_hint_code() -> String {
    "rounded bg-apple-notebook-border px-1 py-0.5 text-apple-notebook-graphite dark:bg-white/10 dark:text-gray-200"
        .to_string()
}

pub fn app_title_text() -> String {
    format!(
        "text-xl font-bold leading-[1.3] {}",
        ThemeText::Primary.classes()
    )
}

pub fn note_title_text() -> &'static str {
    "text-xl font-semibold leading-7 md:text-2xl"
}

pub fn preview_title_text() -> &'static str {
    note_title_text()
}

pub fn empty_state_title_text() -> String {
    format!(
        "text-2xl font-semibold leading-8 {}",
        ThemeText::Primary.classes()
    )
}

pub fn empty_state_body_text() -> &'static str {
    "mt-2 text-sm leading-6"
}

pub fn empty_state_placeholder_text() -> &'static str {
    "text-xl font-semibold leading-7"
}

pub fn note_list_title_text() -> String {
    format!(
        "min-w-0 flex-1 truncate pr-2 text-[13px] font-semibold leading-5 {}",
        ThemeText::Primary.classes()
    )
}

pub fn note_list_meta_row() -> &'static str {
    "mt-0.5 flex space-x-2 text-[11px] leading-4"
}

pub fn editor_body_text() -> &'static str {
    "text-sm leading-6 font-mono"
}

pub fn preview_body_text() -> &'static str {
    "prose prose-sm prose-yellow w-full max-w-[72ch] dark:prose-invert"
}

pub fn note_measure() -> &'static str {
    NOTE_MEASURE_CLASS
}

pub fn pane_inline_inset() -> &'static str {
    PANE_INLINE_INSET_CLASS
}

pub fn pane_top_inset() -> &'static str {
    PANE_TOP_INSET_CLASS
}

pub fn sidebar_footer() -> String {
    format!(
        "{} shrink-0 gap-2 px-3 py-1.5 border-t border-apple-notebook-borderStrong dark:border-apple-notebook-darkBorder flex items-center justify-between text-[11px] leading-4 {}",
        COMPACT_FOOTER_HEIGHT_CLASS,
        ThemeText::Subtle.classes()
    )
}

pub fn backup_footer_label() -> String {
    format!(
        "truncate text-[11px] font-semibold leading-4 {}",
        ThemeText::Primary.classes()
    )
}

pub fn backup_footer_summary() -> String {
    format!(
        "inline-flex min-w-0 items-center gap-1 truncate text-[11px] leading-4 {}",
        ThemeText::Subtle.classes()
    )
}

pub fn backup_footer_status_dot() -> &'static str {
    "h-1.5 w-1.5 shrink-0 rounded-full bg-emerald-500"
}

pub fn backup_import_preview() -> String {
    format!(
        "border-t border-apple-notebook-borderStrong px-4 py-3 text-xs leading-5 dark:border-apple-notebook-darkBorder {}",
        ThemeSurface::EditorChrome.classes()
    )
}

pub fn editor_footer() -> String {
    format!(
        "{} shrink-0 gap-x-2 gap-y-1 border-t border-apple-notebook-border px-4 py-1.5 text-[11px] leading-4 dark:border-apple-notebook-darkBorder flex items-center justify-between {}",
        COMPACT_FOOTER_HEIGHT_CLASS,
        ThemeSurface::EditorChrome.classes()
    )
}

pub fn compact_controls() -> &'static str {
    "flex min-w-0 items-center justify-center gap-x-1"
}

pub fn editor_footer_stats() -> String {
    format!(
        "hidden min-w-0 flex-1 items-center gap-4 truncate sm:flex {}",
        ThemeText::Muted.classes()
    )
}

pub fn editor_footer_mode_group() -> &'static str {
    "flex min-w-0 flex-none items-center justify-center gap-2"
}

pub fn editor_footer_mode_label() -> String {
    format!("hidden sm:inline {}", ThemeText::Muted.classes())
}

pub fn editor_footer_meta() -> String {
    format!(
        "hidden min-w-0 flex-1 items-center justify-end gap-2 sm:flex {}",
        ThemeText::Muted.classes()
    )
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
        "{visibility} h-11 min-w-[2.75rem] items-center justify-center rounded-md px-3 text-xs transition-colors md:h-6 md:min-w-0 md:px-2 md:py-0 md:text-[11px] {state_classes}"
    )
}

pub fn compact_help_button() -> String {
    format!(
        "inline-flex h-11 w-11 items-center justify-center rounded-md px-0 text-xs transition-colors md:h-6 md:w-auto md:px-2 md:py-0 md:text-[11px] {}",
        ThemeState::SegmentedIdle.classes()
    )
}

pub fn compact_icon_button() -> String {
    format!(
        "inline-flex h-6 w-6 items-center justify-center rounded-md transition-colors {}",
        ThemeState::SegmentedIdle.classes()
    )
}

pub fn backup_footer_button() -> String {
    format!(
        "inline-flex h-7 cursor-pointer items-center rounded-md px-2 text-[11px] md:h-auto md:px-1.5 md:py-0.5 {}",
        ThemeState::SegmentedIdle.classes()
    )
}

pub fn danger_footer_button() -> String {
    format!(
        "ml-auto inline-flex h-8 cursor-pointer items-center rounded-md px-2 py-1 text-[11px] md:h-auto md:px-1.5 md:py-0.5 {}",
        ThemeState::DangerMenuItem.classes()
    )
}

pub fn recovery_action_button() -> String {
    format!(
        "inline-flex h-9 items-center rounded-md px-2.5 text-[11px] md:h-auto md:px-1.5 md:py-0.5 {}",
        ThemeState::SegmentedIdle.classes()
    )
}

pub fn recovery_danger_button() -> String {
    format!(
        "inline-flex h-9 items-center rounded-md px-2.5 text-[11px] md:h-auto md:px-1.5 md:py-0.5 {}",
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
    let (border_classes, state_classes) = if is_selected {
        ("border", ThemeState::NoteRowSelected.classes())
    } else {
        (
            "border border-transparent",
            ThemeState::NoteRowIdle.classes(),
        )
    };

    format!(
        "mx-3 mb-1 rounded-md {border_classes} px-2 py-1 cursor-pointer transition-colors duration-200 ease-out group {state_classes}"
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
            assert_eq!(classes.matches(COMPACT_FOOTER_HEIGHT_CLASS).count(), 1);
            assert!(classes.contains("py-1.5"));
            assert!(classes.contains("text-[11px]"));
            assert!(classes.contains("leading-4"));
            assert!(classes.contains("border-t"));
            assert!(classes.contains("dark:border-apple-notebook-darkBorder"));
            assert!(classes.contains("flex"));
            assert!(classes.contains("items-center"));
        }

        assert!(sidebar.contains("border-apple-notebook-borderStrong"));
        assert!(editor.contains("border-apple-notebook-border"));
        assert!(sidebar.contains("px-3"));
        assert!(editor.contains("px-4"));
        assert!(sidebar.contains("justify-between"));
        assert!(!sidebar.contains("flex-wrap"));
        assert!(!editor.contains("flex-wrap"));
        assert!(editor.contains("justify-between"));
        assert!(backup_footer_summary().contains("inline-flex"));
        assert!(backup_footer_status_dot().contains("bg-emerald-500"));
        assert!(editor_footer_stats().contains("sm:flex"));
        assert!(editor_footer_mode_group().contains("justify-center"));
        assert!(editor_footer_meta().contains("justify-end"));
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
        let recovery = recovery_action_button();
        let recovery_danger = recovery_danger_button();

        assert!(controls.contains("gap-x-1"));
        assert!(!controls.contains("gap-y-1"));

        for classes in [&idle, &active, &help] {
            assert!(classes.contains("rounded-md"));
            assert!(classes.contains("h-11"));
            assert!(classes.contains("md:h-6"));
            assert!(classes.contains("md:px-2"));
            assert!(classes.contains("md:py-0"));
            assert!(classes.contains("md:text-[11px]"));
        }

        assert!(idle.contains("min-w-[2.75rem]"));
        assert!(active.contains("min-w-[2.75rem]"));
        assert!(help.contains("w-11"));
        assert!(compact_icon_button().contains("h-6"));
        assert!(compact_icon_button().contains("w-6"));

        for classes in [&backup, &danger, &recovery, &recovery_danger] {
            assert!(classes.contains("rounded-md"));
            assert!(classes.contains("text-[11px]"));
        }

        assert!(split.starts_with("hidden lg:inline-flex"));
        assert!(active.contains("border-apple-notebook-amberBorder"));
        assert!(backup.contains("h-7"));
        assert!(recovery.contains("h-9"));
        assert!(recovery_danger.contains("h-9"));
        assert!(danger.contains("text-red"));
        assert!(recovery_danger.contains("text-red"));
    }

    #[test]
    fn search_hint_tag_and_selected_note_recipes_are_theme_aware() {
        let search = search_hint();
        let search_input = search_input();
        let search_code = search_hint_code();
        let tag = tag_pill();
        let selected = note_row(true);
        let idle = note_row(false);

        assert!(search.contains("absolute"));
        assert!(search.contains("pointer-events-none"));
        assert!(search.contains("top-full"));
        assert!(search.contains("z-30"));
        assert!(search.contains("p-2"));
        assert!(search.contains("shadow-sm"));
        assert!(search.contains("text-apple-notebook-graphite"));
        assert!(search.contains("dark:text-apple-notebook-frame"));
        assert!(search_input.contains("bg-apple-notebook-border"));
        assert!(!search_input.contains(&["bg", "black"].join("-")));
        assert!(search_code.contains("bg-apple-notebook-border"));
        assert!(!search_code.contains(&["bg", "black"].join("-")));

        assert!(tag.contains("rounded-full"));
        assert!(tag.contains("text-xs"));
        assert!(tag.contains("dark:bg-white/10"));

        assert!(selected.contains("border-apple-notebook-amberBorder"));
        assert!(selected.contains("ring-1"));
        assert!(selected.contains("dark:bg-apple-notebook-amber/25"));
        assert!(idle.contains("hover:bg-apple-notebook-border"));
    }

    #[test]
    fn typography_recipes_keep_app_roles_on_the_documented_scale() {
        let app_title = app_title_text();
        let empty_title = empty_state_title_text();
        let note_list_title = note_list_title_text();

        assert!(app_title.contains("text-xl"));
        assert!(app_title.contains("font-bold"));
        assert!(app_title.contains("leading-[1.3]"));
        assert!(note_title_text().contains("text-xl"));
        assert!(note_title_text().contains("md:text-2xl"));
        assert!(note_title_text().contains("font-semibold"));
        assert_eq!(preview_title_text(), note_title_text());

        assert!(empty_title.contains("text-2xl"));
        assert!(empty_title.contains("font-semibold"));
        assert!(empty_title.contains("leading-8"));
        assert!(empty_state_body_text().contains("text-sm"));
        assert!(empty_state_body_text().contains("leading-6"));
        assert!(empty_state_placeholder_text().contains("text-xl"));
        assert!(empty_state_placeholder_text().contains("font-semibold"));

        assert!(note_list_title.contains("text-[13px]"));
        assert!(note_list_title.contains("font-semibold"));
        assert!(note_list_title.contains("leading-5"));
        assert!(note_list_meta_row().contains("text-[11px]"));
        assert!(note_list_meta_row().contains("leading-4"));
        assert!(note_list_meta_row().contains("mt-0.5"));

        assert!(editor_body_text().contains("text-sm"));
        assert!(editor_body_text().contains("leading-6"));
        assert!(editor_body_text().contains("font-mono"));
        assert!(preview_body_text().contains("prose-sm"));
        assert!(preview_body_text().contains("prose-yellow"));
        assert!(preview_body_text().contains("dark:prose-invert"));
        assert!(preview_body_text().contains("max-w-[72ch]"));
        assert_eq!(note_measure(), NOTE_MEASURE_CLASS);
        assert_eq!(pane_inline_inset(), PANE_INLINE_INSET_CLASS);
        assert_eq!(pane_top_inset(), PANE_TOP_INSET_CLASS);
        assert!(pane_top_inset().contains("pt-4"));
        assert!(backup_import_preview().contains("text-xs"));
        assert!(backup_import_preview().contains("leading-5"));
    }
}
