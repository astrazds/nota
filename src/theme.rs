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
            Self::RootApp => {
                "bg-white text-gray-900 dark:bg-apple-dark-bg dark:text-white transition-colors duration-300"
            }
            Self::Sidebar => {
                "bg-apple-gray-100 border-apple-gray-300 dark:bg-apple-dark-sidebar dark:border-apple-dark-border"
            }
            Self::EditorChrome => {
                "bg-white border-apple-gray-200 dark:bg-apple-dark-bg dark:border-apple-dark-border transition-colors"
            }
            Self::WritingSurface => "bg-white dark:bg-apple-dark-bg",
            Self::Preview => {
                "bg-white text-gray-900 prose-yellow dark:bg-apple-dark-bg dark:text-white dark:prose-invert"
            }
            Self::SplitPreview => {
                "bg-apple-gray-100 text-gray-900 prose-yellow border-apple-gray-300 dark:bg-apple-dark-sidebar dark:text-white dark:prose-invert dark:border-apple-dark-border"
            }
            Self::ModalPanel => {
                "bg-white border-apple-gray-200 dark:bg-apple-dark-sidebar dark:border-apple-dark-border"
            }
            Self::ModalChrome => {
                "bg-apple-gray-100 border-apple-gray-200 dark:bg-white/5 dark:border-apple-dark-border"
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
            Self::Primary => "text-gray-900 dark:text-white",
            Self::Muted => "text-gray-500 dark:text-gray-400",
            Self::Subtle => "text-gray-400 dark:text-gray-500",
            Self::Placeholder => "placeholder:text-gray-300 dark:placeholder:text-gray-600",
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
            Self::PrimaryText => "text-apple-yellow hover:text-yellow-600",
            Self::PrimaryFill => "bg-apple-yellow text-white hover:bg-yellow-600",
            Self::Selection => "selection:bg-apple-yellow/30",
            Self::Highlight => "bg-apple-yellow/30",
            Self::Focus => {
                "focus:outline-none focus:ring-2 focus:ring-apple-yellow focus:ring-offset-2 dark:focus:ring-offset-apple-dark-bg"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeState {
    IconButton,
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
            Self::IconButton => "hover:bg-apple-gray-200 dark:hover:bg-white/5 transition-colors",
            Self::SidebarToggle => {
                "text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
            }
            Self::SegmentedIdle => {
                "border bg-white border-gray-200 text-gray-500 hover:border-gray-300 dark:bg-white/5 dark:border-apple-dark-border dark:text-gray-400 dark:hover:border-gray-500"
            }
            Self::SegmentedActive => {
                "border bg-apple-yellow/10 border-apple-yellow text-apple-yellow"
            }
            Self::ToolbarButton => {
                "hover:bg-apple-gray-200 dark:hover:bg-white/5 text-gray-600 dark:text-gray-400"
            }
            Self::NoteRowIdle => {
                "border-apple-gray-200 dark:border-apple-dark-border hover:bg-apple-gray-200 dark:hover:bg-white/5"
            }
            Self::NoteRowSelected => {
                "border-apple-yellow bg-apple-yellow/10 ring-1 ring-apple-yellow/25 dark:border-apple-yellow dark:bg-apple-yellow/20 dark:ring-apple-yellow/25"
            }
            Self::NoteActionMenu => {
                "border-apple-gray-200 bg-white dark:border-apple-dark-border dark:bg-apple-dark-sidebar"
            }
            Self::NoteActionButton => {
                "text-gray-400 hover:text-gray-700 hover:bg-apple-gray-300/70 dark:hover:text-gray-200 dark:hover:bg-white/10"
            }
            Self::NoteMenuItem => {
                "text-gray-700 hover:bg-apple-gray-200 dark:text-gray-200 dark:hover:bg-white/10"
            }
            Self::TagPill => {
                "bg-apple-gray-200/70 text-gray-500 hover:bg-apple-gray-300/70 dark:bg-white/10 dark:text-gray-400 dark:hover:bg-white/20"
            }
            Self::FilterPill => "bg-apple-yellow text-white",
            Self::SecondaryButton => {
                "bg-gray-200 text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20"
            }
            Self::DangerButton => {
                "bg-red-500 text-white hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-400"
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
            ThemeState::IconButton.classes(),
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

        assert!(preview.contains("text-gray-900"));
        assert!(preview.contains("dark:text-white"));
        assert!(preview.contains("dark:prose-invert"));

        assert!(split_preview.contains("bg-apple-gray-100"));
        assert!(split_preview.contains("text-gray-900"));
        assert!(split_preview.contains("dark:bg-apple-dark-sidebar"));
        assert!(split_preview.contains("dark:text-white"));
        assert!(split_preview.contains("dark:prose-invert"));

        assert!(selected_row.contains("bg-apple-yellow/10"));
        assert!(selected_row.contains("ring-1"));
        assert!(selected_row.contains("dark:bg-apple-yellow/20"));
    }
}
