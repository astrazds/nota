#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeSurfaceGroup {
    RootApp,
    Sidebar,
    WritingSurface,
    Preview,
    ModalSurface,
    TextRole,
    MutedText,
    Border,
    Divider,
    Accent,
    Selection,
    Focus,
    Hover,
    Danger,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalUiConcern {
    Layout,
    Spacing,
    Sizing,
    ResponsiveBehavior,
    ComponentStructure,
}

#[cfg(test)]
pub fn load_bearing_theme_groups() -> &'static [ThemeSurfaceGroup] {
    &[
        ThemeSurfaceGroup::RootApp,
        ThemeSurfaceGroup::Sidebar,
        ThemeSurfaceGroup::WritingSurface,
        ThemeSurfaceGroup::Preview,
        ThemeSurfaceGroup::ModalSurface,
        ThemeSurfaceGroup::TextRole,
        ThemeSurfaceGroup::MutedText,
        ThemeSurfaceGroup::Border,
        ThemeSurfaceGroup::Divider,
        ThemeSurfaceGroup::Accent,
        ThemeSurfaceGroup::Selection,
        ThemeSurfaceGroup::Focus,
        ThemeSurfaceGroup::Hover,
        ThemeSurfaceGroup::Danger,
    ]
}

#[cfg(test)]
pub fn local_ui_concerns() -> &'static [LocalUiConcern] {
    &[
        LocalUiConcern::Layout,
        LocalUiConcern::Spacing,
        LocalUiConcern::Sizing,
        LocalUiConcern::ResponsiveBehavior,
        LocalUiConcern::ComponentStructure,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSurface {
    RootApp,
    WorkspaceFrame,
    Sidebar,
    EditorChrome,
    WritingSurface,
    Preview,
    SplitPreview,
    ModalPanel,
    ModalChrome,
}

impl ThemeSurface {
    pub fn classes(self) -> &'static str {
        match self {
            Self::RootApp => "transition-colors duration-300",
            Self::WorkspaceFrame => {
                "border-apple-notebook-border dark:border-apple-notebook-darkBorder"
            }
            Self::Sidebar => {
                "bg-apple-notebook-sidebar border-apple-notebook-borderStrong dark:bg-apple-notebook-darkSidebar dark:border-apple-notebook-darkBorder"
            }
            Self::EditorChrome => {
                "bg-apple-notebook-surface border-apple-notebook-border dark:bg-apple-notebook-darkSurface dark:border-apple-notebook-darkBorder transition-colors"
            }
            Self::WritingSurface => "bg-apple-notebook-surface dark:bg-apple-notebook-darkSurface",
            Self::Preview => {
                "bg-apple-notebook-surface text-apple-notebook-graphite prose-yellow dark:bg-apple-notebook-darkSurface dark:text-apple-notebook-frame dark:prose-invert"
            }
            Self::SplitPreview => {
                "bg-apple-notebook-frame text-apple-notebook-graphite prose-yellow border-apple-notebook-border dark:bg-apple-notebook-darkSidebar dark:text-apple-notebook-frame dark:prose-invert dark:border-apple-notebook-darkBorder"
            }
            Self::ModalPanel => {
                "bg-apple-notebook-surface border-apple-notebook-border dark:bg-apple-notebook-darkSidebar dark:border-apple-notebook-darkBorder"
            }
            Self::ModalChrome => {
                "bg-apple-notebook-sidebar border-apple-notebook-border dark:bg-white/5 dark:border-apple-notebook-darkBorder"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeText {
    Primary,
    Muted,
    Subtle,
    Placeholder,
}

impl ThemeText {
    pub fn classes(self) -> &'static str {
        match self {
            Self::Primary => "text-apple-notebook-graphite dark:text-apple-notebook-frame",
            Self::Muted => "text-apple-notebook-muted dark:text-gray-400",
            Self::Subtle => "text-gray-400 dark:text-gray-500",
            Self::Placeholder => "placeholder:text-gray-400 dark:placeholder:text-gray-600",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeAccent {
    PrimaryText,
    PrimaryFill,
    Selection,
    Highlight,
    Focus,
}

impl ThemeAccent {
    pub fn classes(self) -> &'static str {
        match self {
            Self::PrimaryText => {
                "text-amber-700 hover:text-amber-800 dark:text-apple-notebook-amber dark:hover:text-amber-300"
            }
            Self::PrimaryFill => {
                "bg-apple-yellow text-apple-notebook-graphite hover:bg-apple-notebook-amber"
            }
            Self::Selection => "selection:bg-apple-notebook-selected",
            Self::Highlight => "bg-apple-notebook-selected",
            Self::Focus => {
                "focus:outline-none focus:ring-2 focus:ring-apple-notebook-amber focus:ring-offset-2 dark:focus:ring-offset-apple-notebook-darkSurface"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeState {
    SidebarToggle,
    SegmentedIdle,
    SegmentedActive,
    ToolbarButton,
    NoteRowIdle,
    NoteRowSelected,
    NoteActionMenu,
    NoteActionButton,
    NoteMenuItem,
    TagPill,
    FilterPill,
    SecondaryButton,
    DangerButton,
    DangerMenuItem,
    EmptyState,
    EmptyIllustration,
}

impl ThemeState {
    pub fn classes(self) -> &'static str {
        match self {
            Self::SidebarToggle => {
                "text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
            }
            Self::SegmentedIdle => {
                "border bg-apple-notebook-surface border-apple-notebook-border text-apple-notebook-muted hover:border-apple-notebook-borderStrong dark:bg-white/5 dark:border-apple-notebook-darkBorder dark:text-gray-400 dark:hover:border-gray-500"
            }
            Self::SegmentedActive => {
                "border bg-apple-notebook-selected border-apple-notebook-amberBorder text-amber-800 dark:bg-apple-notebook-amber/20 dark:border-apple-notebook-amber dark:text-apple-notebook-amber"
            }
            Self::ToolbarButton => {
                "hover:bg-apple-notebook-border dark:hover:bg-white/5 text-apple-notebook-muted dark:text-gray-400"
            }
            Self::NoteRowIdle => {
                "border-apple-notebook-border dark:border-apple-notebook-darkBorder hover:bg-apple-notebook-border dark:hover:bg-white/5"
            }
            Self::NoteRowSelected => {
                "border-apple-notebook-amberBorder bg-apple-notebook-selected ring-1 ring-apple-notebook-amber/30 dark:border-apple-notebook-amber dark:bg-apple-notebook-amber/25 dark:ring-apple-notebook-amber/25"
            }
            Self::NoteActionMenu => {
                "border-apple-notebook-border bg-apple-notebook-surface dark:border-apple-notebook-darkBorder dark:bg-apple-notebook-darkSidebar"
            }
            Self::NoteActionButton => {
                "text-gray-400 hover:text-apple-notebook-graphite hover:bg-apple-notebook-border dark:hover:text-gray-200 dark:hover:bg-white/10"
            }
            Self::NoteMenuItem => {
                "text-apple-notebook-graphite hover:bg-apple-notebook-border dark:text-gray-200 dark:hover:bg-white/10"
            }
            Self::TagPill => {
                "bg-apple-notebook-border text-apple-notebook-muted hover:bg-apple-notebook-borderStrong dark:bg-white/10 dark:text-gray-400 dark:hover:bg-white/20"
            }
            Self::FilterPill => "bg-apple-notebook-amber text-apple-notebook-graphite",
            Self::SecondaryButton => {
                "bg-apple-notebook-border text-apple-notebook-graphite hover:bg-apple-notebook-borderStrong focus:outline-none focus:ring-2 focus:ring-apple-notebook-amber dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20"
            }
            Self::DangerButton => {
                "bg-red-600 text-apple-notebook-surface hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
            }
            Self::DangerMenuItem => {
                "text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-500/10"
            }
            Self::EmptyState => "text-gray-500 dark:text-gray-400",
            Self::EmptyIllustration => "text-apple-yellow",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_surface_groups_are_semantic_and_layout_stays_local() {
        let groups = load_bearing_theme_groups();

        assert!(groups.contains(&ThemeSurfaceGroup::RootApp));
        assert!(groups.contains(&ThemeSurfaceGroup::Sidebar));
        assert!(groups.contains(&ThemeSurfaceGroup::WritingSurface));
        assert!(groups.contains(&ThemeSurfaceGroup::Preview));
        assert!(groups.contains(&ThemeSurfaceGroup::ModalSurface));
        assert!(groups.contains(&ThemeSurfaceGroup::MutedText));
        assert!(groups.contains(&ThemeSurfaceGroup::Border));
        assert!(groups.contains(&ThemeSurfaceGroup::Accent));
        assert!(groups.contains(&ThemeSurfaceGroup::Selection));
        assert!(groups.contains(&ThemeSurfaceGroup::Focus));
        assert!(groups.contains(&ThemeSurfaceGroup::Hover));
        assert!(groups.contains(&ThemeSurfaceGroup::Danger));

        let local_concerns = local_ui_concerns();
        assert!(local_concerns.contains(&LocalUiConcern::Layout));
        assert!(local_concerns.contains(&LocalUiConcern::Spacing));
        assert!(local_concerns.contains(&LocalUiConcern::Sizing));
        assert!(local_concerns.contains(&LocalUiConcern::ResponsiveBehavior));
        assert!(local_concerns.contains(&LocalUiConcern::ComponentStructure));
    }

    #[test]
    fn semantic_theme_recipes_do_not_absorb_layout_spacing_or_responsive_tokens() {
        let semantic_recipes = [
            ThemeSurface::RootApp.classes(),
            ThemeSurface::WorkspaceFrame.classes(),
            ThemeSurface::Sidebar.classes(),
            ThemeSurface::EditorChrome.classes(),
            ThemeSurface::WritingSurface.classes(),
            ThemeSurface::Preview.classes(),
            ThemeSurface::SplitPreview.classes(),
            ThemeSurface::ModalPanel.classes(),
            ThemeSurface::ModalChrome.classes(),
            ThemeText::Primary.classes(),
            ThemeText::Muted.classes(),
            ThemeText::Subtle.classes(),
            ThemeText::Placeholder.classes(),
            ThemeAccent::PrimaryText.classes(),
            ThemeAccent::PrimaryFill.classes(),
            ThemeAccent::Selection.classes(),
            ThemeAccent::Highlight.classes(),
            ThemeAccent::Focus.classes(),
            ThemeState::SidebarToggle.classes(),
            ThemeState::SegmentedIdle.classes(),
            ThemeState::SegmentedActive.classes(),
            ThemeState::ToolbarButton.classes(),
            ThemeState::NoteRowIdle.classes(),
            ThemeState::NoteRowSelected.classes(),
            ThemeState::NoteActionMenu.classes(),
            ThemeState::NoteActionButton.classes(),
            ThemeState::NoteMenuItem.classes(),
            ThemeState::TagPill.classes(),
            ThemeState::FilterPill.classes(),
            ThemeState::SecondaryButton.classes(),
            ThemeState::DangerButton.classes(),
            ThemeState::DangerMenuItem.classes(),
            ThemeState::EmptyState.classes(),
            ThemeState::EmptyIllustration.classes(),
        ];

        for recipe in semantic_recipes {
            for token in recipe.split_whitespace() {
                assert!(
                    !is_local_layout_token(token),
                    "theme recipe should not own local UI token `{token}` in `{recipe}`"
                );
            }
        }
    }

    fn is_local_layout_token(token: &str) -> bool {
        matches!(
            token,
            "flex"
                | "grid"
                | "block"
                | "inline-flex"
                | "fixed"
                | "absolute"
                | "relative"
                | "sticky"
                | "overflow-hidden"
                | "overflow-y-auto"
        ) || token.starts_with("p-")
            || token.starts_with("px-")
            || token.starts_with("py-")
            || token.starts_with("pt-")
            || token.starts_with("pb-")
            || token.starts_with("pl-")
            || token.starts_with("pr-")
            || token.starts_with("m-")
            || token.starts_with("mx-")
            || token.starts_with("my-")
            || token.starts_with("mt-")
            || token.starts_with("mb-")
            || token.starts_with("ml-")
            || token.starts_with("mr-")
            || token.starts_with("w-")
            || token.starts_with("h-")
            || token.starts_with("min-")
            || token.starts_with("max-")
            || token.starts_with("gap-")
            || token.starts_with("space-")
            || token.starts_with("z-")
            || token.starts_with("inset-")
            || token.starts_with("top-")
            || token.starts_with("right-")
            || token.starts_with("bottom-")
            || token.starts_with("left-")
            || token.starts_with("sm:")
            || token.starts_with("md:")
            || token.starts_with("lg:")
    }

    #[test]
    fn preview_surfaces_and_selection_have_explicit_theme_contrast() {
        let preview = ThemeSurface::Preview.classes();
        let split_preview = ThemeSurface::SplitPreview.classes();
        let selected_row = ThemeState::NoteRowSelected.classes();
        let primary_fill = ThemeAccent::PrimaryFill.classes();
        let danger_button = ThemeState::DangerButton.classes();
        let white_text = ["text", "white"].join("-");

        assert!(preview.contains("text-apple-notebook-graphite"));
        assert!(preview.contains("dark:text-apple-notebook-frame"));
        assert!(preview.contains("dark:prose-invert"));

        assert!(split_preview.contains("bg-apple-notebook-frame"));
        assert!(split_preview.contains("text-apple-notebook-graphite"));
        assert!(split_preview.contains("dark:bg-apple-notebook-darkSidebar"));
        assert!(split_preview.contains("dark:text-apple-notebook-frame"));
        assert!(split_preview.contains("dark:prose-invert"));

        assert!(selected_row.contains("bg-apple-notebook-selected"));
        assert!(selected_row.contains("ring-1"));
        assert!(selected_row.contains("dark:bg-apple-notebook-amber/25"));

        assert!(primary_fill.contains("text-apple-notebook-graphite"));
        assert!(!primary_fill.contains(&white_text));
        assert!(danger_button.contains("text-apple-notebook-surface"));
        assert!(!danger_button.contains(&white_text));
    }
}
