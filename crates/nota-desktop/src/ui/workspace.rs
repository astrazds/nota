use super::dialogs::show_markdown_help;
use super::note_list::NoteLists;
use super::style::{install_css, install_workspace_fonts};
use super::writing_plane::WritingPlane;
use nota_app::app::{AppModel, AppMsg, NotificationTone, SaveStatus};
use nota_app::storage::NativeRecovery;
use nota_core::backup::BackupHealth;
use nota_core::editor_view::EditorViewMode;
use nota_core::markdown_editing::{MarkdownCommand, apply_markdown_command};
use nota_core::note_list_interaction::NoteListDisplayState;
use nota_desktop::selection::gtk_character_range_to_byte_selection;
use nota_desktop::visual_contract::NATIVE_VISUAL_CONTRACT;
#[cfg(feature = "preview-webkit")]
use nota_desktop::webkit_preview::SecurePreview;
use relm4::gtk::prelude::*;
use relm4::{RelmWidgetExt, gtk};
use std::cell::Cell;
use std::rc::Rc;

pub(super) struct DesktopWidgets {
    root: gtk::Box,
    sidebar: gtk::Box,
    editor: gtk::Box,
    editor_header: gtk::Box,
    editor_tools: gtk::Box,
    editor_empty: gtk::Box,
    formatting: gtk::Box,
    footer_row: gtk::CenterBox,
    footer_saved_row: gtk::Box,
    saved_state: gtk::Box,
    modes: gtk::Box,
    editor_navigation: gtk::Button,
    scrim: gtk::Button,
    previous_focus: gtk::glib::WeakRef<gtk::Widget>,
    deleted_count: gtk::Label,
    metadata: gtk::Label,
    pin: gtk::Button,
    pin_menu: gtk::Button,
    note_actions: gtk::MenuButton,
    menus: Vec<gtk::Popover>,
    all_filter: gtk::Button,
    pinned_filter: gtk::Button,
    title: gtk::Entry,
    tags: gtk::Entry,
    content: gtk::TextView,
    status: gtk::Label,
    statistics: gtk::Label,
    notes_count: gtk::Label,
    empty_state: gtk::Label,
    notification: gtk::Label,
    recovery_panel: gtk::Box,
    recovery_actions: gtk::Box,
    restore_previous: gtk::Button,
    create: gtk::Button,
    search: gtk::SearchEntry,
    backup_dot: gtk::Label,
    backup_label: gtk::Label,
    tag_suggestions: gtk::Box,
    edit_tags: gtk::Button,
    cleanup_tags: gtk::Button,
    filter_chip: gtk::Button,
    clear_all: gtk::Button,
    theme_label: gtk::Label,
    writing: gtk::Box,
    content_scroll: gtk::ScrolledWindow,
    surface_scroll: gtk::ScrolledWindow,
    surface_row: gtk::Box,
    #[cfg(feature = "preview-webkit")]
    preview: SecurePreview,
    #[cfg(not(feature = "preview-webkit"))]
    preview_fallback: gtk::Box,
    tag_suggestion_count: Rc<Cell<usize>>,
    mode_buttons: Vec<(EditorViewMode, gtk::Button)>,
    refreshing: Rc<Cell<bool>>,
    rendered_note: Rc<Cell<Option<uuid::Uuid>>>,
}

impl DesktopWidgets {
    pub(super) fn new(
        window: &gtk::ApplicationWindow,
        note_lists: &NoteLists,
        recovery: Option<&NativeRecovery>,
        sender: &relm4::Sender<AppMsg>,
    ) -> Self {
        install_workspace_fonts(window);
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::IconTheme::for_display(&display).add_resource_path("/net/astrazds/Nota/icons");
        }
        let writing_plane_max_px = NATIVE_VISUAL_CONTRACT.editor_measure_px;
        let rendered_note = Rc::new(Cell::new(None::<uuid::Uuid>));
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_css_classes(&["nota-root"]);
        window.add_css_class("nota-root");

        let topbar = gtk::CenterBox::new();
        topbar.set_css_classes(&["nota-topbar"]);
        let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 13);
        let editor_navigation = icon_label_button("nota-panel-left-symbolic", "Notes");
        editor_navigation.set_tooltip_text(Some("Open notes (Ctrl+F to search)"));
        editor_navigation.set_css_classes(&["nota-navigation-button"]);
        let create = icon_button("nota-plus-symbolic", "New note (Ctrl+N)");
        navigation.append(&editor_navigation);
        navigation.append(&create);
        topbar.set_start_widget(Some(&navigation));
        let brand = gtk::Label::new(Some("nota"));
        brand.set_css_classes(&["nota-app-title"]);
        topbar.set_center_widget(Some(&brand));
        let window_actions = gtk::Box::new(gtk::Orientation::Horizontal, 13);
        let settings = gtk::MenuButton::builder()
            .icon_name("nota-sliders-horizontal-symbolic")
            .tooltip_text("Settings")
            .css_classes(["nota-icon-menu"])
            .build();
        let settings_menu = gtk::Popover::new();
        settings_menu.set_css_classes(&["nota-note-actions-popover"]);
        let settings_items = gtk::Box::new(gtk::Orientation::Vertical, 2);
        settings_items.set_css_classes(&["nota-note-menu"]);
        let theme = gtk::Button::new();
        let theme_label = gtk::Label::new(Some("Dark theme"));
        theme_label.set_xalign(0.0);
        theme.set_child(Some(&theme_label));
        let help = gtk::Button::with_label("Markdown help");
        settings_items.append(&theme);
        settings_items.append(&help);
        settings_menu.set_child(Some(&settings_items));
        settings.set_popover(Some(&settings_menu));
        window_actions.append(&settings);
        window_actions.append(&gtk::WindowControls::new(gtk::PackType::End));
        topbar.set_end_widget(Some(&window_actions));
        let window_handle = gtk::WindowHandle::new();
        window_handle.set_css_classes(&["nota-window-handle"]);
        window_handle.set_child(Some(&topbar));
        window.set_titlebar(Some(&window_handle));

        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar.set_width_request(NATIVE_VISUAL_CONTRACT.sidebar_width);
        sidebar.set_css_classes(&["nota-sidebar"]);
        sidebar.set_halign(gtk::Align::Start);
        sidebar.set_vexpand(true);
        sidebar.set_visible(false);

        let sidebar_header = gtk::Box::new(gtk::Orientation::Vertical, 12);
        sidebar_header.set_css_classes(&["nota-sidebar-header"]);
        let identity = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let app_title = gtk::Label::new(Some("Notes"));
        app_title.set_hexpand(true);
        app_title.set_halign(gtk::Align::Start);
        let sidebar_navigation = icon_button("nota-x-symbolic", "Close notes");
        identity.append(&app_title);
        identity.append(&sidebar_navigation);
        sidebar_header.append(&identity);

        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search notes")
            .accessible_role(gtk::AccessibleRole::SearchBox)
            .build();
        search.set_css_classes(&["nota-search"]);
        let search_hint = gtk::Popover::new();
        search_hint.set_parent(&search);
        search_hint.set_autohide(false);
        search_hint.set_has_arrow(true);
        search_hint.set_position(gtk::PositionType::Bottom);
        let hint_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        hint_box.set_margin_start(8);
        hint_box.set_margin_end(8);
        hint_box.set_margin_top(6);
        hint_box.set_margin_bottom(6);
        let hint_title = gtk::Label::new(Some("Syntax"));
        hint_title.set_halign(gtk::Align::Start);
        hint_title.set_css_classes(&["nota-footer-label"]);
        let hint_copy = gtk::Label::new(Some("\"phrase\"   title:plan   tag:work   is:pinned"));
        hint_copy.set_halign(gtk::Align::Start);
        hint_copy.set_wrap(true);
        hint_box.append(&hint_title);
        hint_box.append(&hint_copy);
        search_hint.set_child(Some(&hint_box));
        let filter_chip = gtk::Button::with_label("#tag");
        filter_chip.set_tooltip_text(Some("Clear tag filter"));
        filter_chip.set_css_classes(&["nota-filter-button", "active"]);
        filter_chip.set_visible(false);
        sidebar_header.append(&search);
        let discovery_filters = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        discovery_filters.set_css_classes(&["nota-discovery-filters"]);
        let all_filter = gtk::Button::with_label("All notes");
        let pinned_filter = gtk::Button::with_label("Pinned");
        for (button, pinned_only) in [(&all_filter, false), (&pinned_filter, true)] {
            button.set_css_classes(&["nota-filter-button"]);
            let filter_sender = sender.clone();
            button.connect_clicked(move |_| {
                let _send_result = filter_sender.send(AppMsg::ClearTag);
                let _send_result = filter_sender.send(AppMsg::SetPinnedFilter(pinned_only));
            });
            discovery_filters.append(button);
        }
        discovery_filters.append(&filter_chip);
        sidebar_header.append(&discovery_filters);
        sidebar.append(&sidebar_header);

        let sidebar_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();
        sidebar_scroll.set_css_classes(&["nota-sidebar-scroll"]);
        let sidebar_content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar_content.set_vexpand(true);
        let notes_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        notes_header.set_css_classes(&["nota-section-header"]);
        let notes_label = gtk::Label::new(Some("YOUR NOTES"));
        notes_label.set_hexpand(true);
        notes_label.set_halign(gtk::Align::Start);
        let notes_count = gtk::Label::new(None);
        notes_count.set_css_classes(&["nota-section-count"]);
        notes_header.append(&notes_label);
        notes_header.append(&notes_count);
        sidebar_content.append(&notes_header);
        let empty_state = gtk::Label::new(Some("No notes yet."));
        empty_state.set_xalign(0.0);
        empty_state.set_wrap(true);
        empty_state.set_css_classes(&["nota-empty-state"]);
        sidebar_content.append(&empty_state);
        sidebar_content.append(note_lists.notes_widget());

        let data_actions = gtk::Box::new(gtk::Orientation::Vertical, 2);
        data_actions.set_css_classes(&["nota-note-menu"]);
        let export_backup = gtk::Button::with_label("Export notes backup");
        let import_backup = gtk::Button::with_label("Import notes backup");
        let export_transition = gtk::Button::with_label("Export complete notebook");
        let import_transition = gtk::Button::with_label("Restore complete notebook");
        import_transition.set_tooltip_text(Some(
            "Restore a desktop transition into an Empty Collection",
        ));
        data_actions.append(&export_backup);
        data_actions.append(&import_backup);
        data_actions.append(&export_transition);
        data_actions.append(&import_transition);

        let deleted_label = gtk::Label::new(Some("Recently deleted"));
        deleted_label.set_halign(gtk::Align::Start);
        deleted_label.set_hexpand(true);
        let deleted_header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        deleted_header.set_css_classes(&["nota-deleted-header"]);
        let deleted_count = gtk::Label::new(Some("0"));
        deleted_header.append(&deleted_label);
        deleted_header.append(&deleted_count);
        let deleted_expander = gtk::Expander::new(None);
        deleted_expander.set_label_widget(Some(&deleted_header));
        deleted_expander.set_expanded(false);
        deleted_expander.set_css_classes(&["nota-deleted-expander"]);
        let clear_all = gtk::Button::with_label("Clear All");
        clear_all.set_halign(gtk::Align::End);
        clear_all.set_css_classes(&["nota-small-button", "danger"]);
        let deleted_panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        deleted_panel.set_css_classes(&["nota-deleted-panel"]);
        deleted_panel.append(note_lists.deleted_widget());
        deleted_panel.append(&clear_all);
        deleted_expander.set_child(Some(&deleted_panel));
        let list_space = gtk::Box::new(gtk::Orientation::Vertical, 0);
        list_space.set_vexpand(true);
        sidebar_content.append(&list_space);
        sidebar_content.append(&deleted_expander);
        sidebar_scroll.set_child(Some(&sidebar_content));
        sidebar.append(&sidebar_scroll);

        let sidebar_footer = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        sidebar_footer.set_height_request(NATIVE_VISUAL_CONTRACT.footer_height);
        sidebar_footer.set_css_classes(&["nota-sidebar-footer"]);
        let backup_status = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let backup_dot = gtk::Label::new(Some("●"));
        backup_dot.set_css_classes(&["nota-backup-dot"]);
        let backup_label = gtk::Label::new(Some("No recent backup"));
        backup_label.set_css_classes(&["nota-footer-label"]);
        backup_status.append(&backup_dot);
        backup_status.append(&gtk::Label::new(Some("Backup")));
        let backup_menu = gtk::Popover::new();
        backup_menu.set_css_classes(&["nota-note-actions-popover"]);
        data_actions.prepend(&backup_label);
        backup_menu.set_child(Some(&data_actions));
        let backup = gtk::MenuButton::new();
        backup.set_css_classes(&["nota-backup-menu"]);
        backup.set_child(Some(&backup_status));
        backup.set_popover(Some(&backup_menu));
        backup.set_hexpand(true);
        backup.set_halign(gtk::Align::Start);
        let diagnostics = gtk::Button::with_label("About Nota");
        diagnostics.set_css_classes(&["nota-footer-button"]);
        sidebar_footer.append(&backup);
        sidebar_footer.append(&diagnostics);
        sidebar.append(&sidebar_footer);

        let editor = gtk::Box::new(gtk::Orientation::Vertical, 0);
        editor.set_hexpand(true);
        editor.set_css_classes(&["nota-editor"]);

        let editor_tools = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        editor_tools.set_css_classes(&["nota-editor-tools"]);
        editor_tools.set_halign(gtk::Align::End);
        let pin = icon_button("nota-pin-symbolic", "Pin note");
        let note_actions = gtk::MenuButton::builder()
            .icon_name("nota-ellipsis-symbolic")
            .tooltip_text("Note actions")
            .css_classes(["nota-icon-menu"])
            .build();
        let note_menu = gtk::Popover::new();
        note_menu.set_has_arrow(false);
        note_menu.set_offset(0, -21);
        note_menu.set_css_classes(&["nota-note-actions-popover"]);
        let note_items = gtk::Box::new(gtk::Orientation::Vertical, 2);
        note_items.set_css_classes(&["nota-note-menu"]);
        let pin_menu = gtk::Button::with_label("Pin note");
        let edit_tags = gtk::Button::with_label("Edit tags");
        let delete = gtk::Button::with_label("Move to recently deleted");
        delete.add_css_class("destructive-action");
        note_items.append(&pin_menu);
        note_items.append(&edit_tags);
        note_items.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        note_items.append(&delete);
        note_menu.set_child(Some(&note_items));
        note_actions.set_popover(Some(&note_menu));
        editor_tools.append(&pin);
        editor_tools.append(&note_actions);
        editor.append(&editor_tools);

        let editor_header = gtk::Box::new(gtk::Orientation::Vertical, 4);
        editor_header.set_css_classes(&["nota-editor-header"]);

        let title = gtk::Entry::builder()
            .placeholder_text("Untitled note")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        title.set_css_classes(&["nota-title"]);
        title.set_hexpand(true);
        title.set_halign(gtk::Align::Fill);
        title.set_truncate_multiline(true);
        let tags = gtk::Entry::builder()
            .placeholder_text("Add tags, separated by commas")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        tags.set_css_classes(&["nota-tags"]);
        tags.set_hexpand(true);
        tags.set_halign(gtk::Align::Fill);
        let header_inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        header_inner.set_hexpand(true);
        header_inner.set_halign(gtk::Align::Fill);
        header_inner.append(&title);
        let metadata = gtk::Label::new(None);
        metadata.set_css_classes(&["nota-metadata"]);
        metadata.set_xalign(0.0);
        metadata.set_wrap(true);
        let metadata_sender = sender.clone();
        metadata.connect_activate_link(move |_, uri| {
            if uri == "nota:edit-tags" {
                let _send_result = metadata_sender.send(AppMsg::StartEditTags);
            } else if let Some(query) = uri.strip_prefix("nota:tag?")
                && let Some((_, tag)) =
                    url::form_urlencoded::parse(query.as_bytes()).find(|(key, _)| key == "tag")
            {
                let _send_result = metadata_sender.send(AppMsg::SetPinnedFilter(false));
                let _send_result = metadata_sender.send(AppMsg::SelectTag(tag.into_owned()));
            }
            gtk::glib::Propagation::Stop
        });
        header_inner.append(&metadata);
        header_inner.append(&tags);
        let cleanup_tags = gtk::Button::with_label("Review Tag cleanup");
        cleanup_tags.set_halign(gtk::Align::Start);
        cleanup_tags.set_css_classes(&["nota-footer-button"]);
        let cleanup_sender = sender.clone();
        cleanup_tags.connect_clicked(move |_| {
            let _send_result = cleanup_sender.send(AppMsg::RequestTagCleanup);
        });
        header_inner.append(&cleanup_tags);
        let tag_suggestions = gtk::Box::new(gtk::Orientation::Vertical, 0);
        tag_suggestions.set_css_classes(&["nota-tag-suggestions"]);
        tag_suggestions.set_halign(gtk::Align::Fill);
        tag_suggestions.set_valign(gtk::Align::Start);
        tag_suggestions.set_hexpand(true);
        tag_suggestions.set_vexpand(false);
        tag_suggestions.set_visible(false);
        header_inner.append(&tag_suggestions);
        tags.set_visible(false);
        let header_plane = WritingPlane::new(writing_plane_max_px);
        header_plane.set_child(Some(&header_inner));
        editor_header.append(&header_plane);
        editor.append(&editor_header);

        let content = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::WordChar)
            .hexpand(true)
            .vexpand(true)
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        content.set_css_classes(&["nota-writing-surface"]);
        content.set_left_margin(0);
        content.set_right_margin(0);
        content.set_top_margin(0);
        content.set_bottom_margin(45);

        let formatting = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        formatting.set_css_classes(&["nota-formatting-toolbar"]);
        let formatting_inner = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        let bold = formatting_button("B", "Bold");
        bold.add_css_class("nota-format-bold");
        let italic = formatting_button("I", "Italic");
        italic.add_css_class("nota-format-italic");
        let heading = formatting_button("H₂", "Heading");
        let bullet_list = icon_button("nota-list-symbolic", "Bullet list");
        let link = icon_button("nota-link-symbolic", "Link");
        let strike = gtk::Button::with_label("Strikethrough");
        let task = icon_button("nota-list-checks-symbolic", "Task list");
        let table = gtk::Button::with_label("Insert table");
        formatting_inner.append(&bold);
        formatting_inner.append(&italic);
        formatting_inner.append(&heading);
        let formatting_divider = gtk::Separator::new(gtk::Orientation::Vertical);
        formatting_divider.set_css_classes(&["nota-formatting-divider"]);
        formatting_inner.append(&formatting_divider);
        formatting_inner.append(&bullet_list);
        formatting_inner.append(&task);
        formatting_inner.append(&link);
        note_items.insert_child_after(&strike, Some(&edit_tags));
        note_items.insert_child_after(&table, Some(&strike));
        for button in [
            &theme,
            &help,
            &export_backup,
            &import_backup,
            &export_transition,
            &import_transition,
            &pin_menu,
            &edit_tags,
            &strike,
            &table,
            &delete,
        ] {
            if let Some(label) = button
                .child()
                .and_then(|child| child.downcast::<gtk::Label>().ok())
            {
                label.set_xalign(0.0);
            }
        }
        let formatting_row = WritingPlane::new(writing_plane_max_px + 12);
        formatting_row.set_child(Some(&formatting_inner));
        formatting.append(&formatting_row);
        editor.append(&formatting);

        let status = gtk::Label::new(Some("Saved locally"));
        status.set_halign(gtk::Align::End);
        status.set_css_classes(&["nota-save-status"]);
        let saved_state = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        saved_state.set_valign(gtk::Align::Center);
        let saved_dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        saved_dot.set_css_classes(&["nota-save-dot"]);
        saved_dot.set_valign(gtk::Align::Center);
        saved_state.append(&saved_dot);
        saved_state.append(&status);
        let statistics = gtk::Label::new(Some("0 words"));
        statistics.set_halign(gtk::Align::Start);
        statistics.set_css_classes(&["nota-statistics"]);
        let notification = gtk::Label::new(None);
        notification.set_halign(gtk::Align::End);
        notification.set_valign(gtk::Align::Start);
        notification.set_margin_top(12);
        notification.set_margin_end(16);
        notification.set_wrap(true);
        notification.set_accessible_role(gtk::AccessibleRole::Status);
        notification.set_css_classes(&["nota-notification"]);

        let recovery_panel = gtk::Box::new(gtk::Orientation::Vertical, 10);
        recovery_panel.set_css_classes(&["nota-recovery"]);
        recovery_panel.set_margin_top(20);
        recovery_panel.set_margin_start(32);
        recovery_panel.set_margin_end(32);
        let recovery_title = gtk::Label::new(Some("Nota could not read the saved collection"));
        recovery_title.set_wrap(true);
        recovery_title.update_property(&[gtk::accessible::Property::Label("Storage Recovery")]);
        let recovery_copy = gtk::Label::new(Some(
            "Restore the Previous Snapshot, start empty, or Import Backup. Editing stays blocked until you choose a path.",
        ));
        recovery_copy.set_wrap(true);
        let recovery_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let restore_previous = gtk::Button::with_label("Restore Previous Snapshot");
        restore_previous.set_sensitive(
            recovery
                .as_ref()
                .and_then(|recovery| recovery.previous_snapshot.as_ref())
                .is_some(),
        );
        let start_empty = gtk::Button::with_label("Start Empty");
        let recovery_import = gtk::Button::with_label("Import Backup");
        recovery_actions.append(&restore_previous);
        recovery_actions.append(&start_empty);
        recovery_actions.append(&recovery_import);
        recovery_panel.append(&recovery_title);
        recovery_panel.append(&recovery_copy);
        recovery_panel.append(&recovery_actions);

        let writing = gtk::Box::new(gtk::Orientation::Vertical, 0);
        writing.set_hexpand(true);
        writing.set_vexpand(true);
        writing.set_css_classes(&["nota-writing"]);
        let body_plane = WritingPlane::new(writing_plane_max_px);
        body_plane.set_margin_start(42);
        body_plane.set_margin_end(42);
        body_plane.set_margin_top(31);
        body_plane.set_vexpand(true);
        body_plane.set_child(Some(&content));
        let content_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .child(&body_plane)
            .build();
        content_scroll.set_css_classes(&["nota-content-scroll"]);
        writing.append(&content_scroll);

        let surface_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        surface_row.set_hexpand(true);
        surface_row.set_vexpand(true);
        surface_row.set_homogeneous(false);
        surface_row.set_css_classes(&["nota-surface-row"]);
        editor.append(&recovery_panel);
        let editor_empty = gtk::Box::new(gtk::Orientation::Vertical, 12);
        editor_empty.set_css_classes(&["nota-editor-empty"]);
        editor_empty.set_halign(gtk::Align::Center);
        editor_empty.set_valign(gtk::Align::Center);
        editor_empty.set_vexpand(true);
        let empty_heading = gtk::Label::new(Some("A quiet place for your notes"));
        empty_heading.set_css_classes(&["nota-empty-title"]);
        let empty_hint = gtk::Label::new(Some("Create a note to start writing."));
        empty_hint.set_css_classes(&["nota-empty-copy"]);
        let empty_capture = gtk::Button::with_label("New note");
        empty_capture.set_css_classes(&["nota-footer-button"]);
        let capture_sender = sender.clone();
        empty_capture.connect_clicked(move |_| {
            let _send_result = capture_sender.send(AppMsg::QuickCapture);
        });
        editor_empty.append(&empty_heading);
        editor_empty.append(&empty_hint);
        editor_empty.append(&empty_capture);
        editor.append(&editor_empty);
        let surface_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .hexpand(true)
            .vexpand(true)
            .child(&surface_row)
            .build();
        editor.append(&surface_scroll);
        surface_row.append(&writing);
        #[cfg(feature = "preview-webkit")]
        let preview = {
            let link_sender = sender.clone();
            let preview = SecurePreview::new(move |target| {
                if let Err(error) = gtk::gio::AppInfo::launch_default_for_uri(
                    target.as_str(),
                    None::<&gtk::gio::AppLaunchContext>,
                ) {
                    let _send_result = link_sender.send(AppMsg::OperationFailed(format!(
                        "Could not open link: {error}"
                    )));
                }
            });
            let preview_plane = WritingPlane::new(writing_plane_max_px);
            preview_plane.set_vexpand(true);
            preview_plane.set_margin_start(42);
            preview_plane.set_margin_end(42);
            preview_plane.set_margin_top(31);
            preview.widget().set_hexpand(true);
            preview.widget().set_vexpand(true);
            preview_plane.set_child(Some(preview.widget()));
            surface_row.append(&preview_plane);
            preview
        };
        #[cfg(not(feature = "preview-webkit"))]
        let preview_fallback = {
            let preview_fallback = gtk::Box::new(gtk::Orientation::Vertical, 8);
            preview_fallback.set_hexpand(true);
            preview_fallback.set_vexpand(true);
            preview_fallback.set_margin_start(32);
            preview_fallback.set_margin_end(32);
            preview_fallback.set_margin_top(24);
            preview_fallback.set_css_classes(&["nota-preview-fallback"]);
            preview_fallback.set_visible(false);
            let heading = gtk::Label::new(Some("Preview is unavailable in this build"));
            heading.set_halign(gtk::Align::Start);
            heading.set_wrap(true);
            heading.set_css_classes(&["nota-empty-title"]);
            let copy = gtk::Label::new(Some(
                "Write mode still works. Rebuild with the preview-webkit feature (WebKitGTK 6) for rendered Markdown Preview and Split.",
            ));
            copy.set_halign(gtk::Align::Start);
            copy.set_wrap(true);
            preview_fallback.append(&heading);
            preview_fallback.append(&copy);
            surface_row.append(&preview_fallback);
            preview_fallback
        };

        let editor_footer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        editor_footer.set_height_request(NATIVE_VISUAL_CONTRACT.footer_height);
        editor_footer.set_css_classes(&["nota-editor-footer"]);
        let footer_row = gtk::CenterBox::new();
        footer_row.set_start_widget(Some(&statistics));
        let modes = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        modes.set_css_classes(&["nota-mode-group"]);
        let mut mode_buttons = Vec::new();
        for (label, mode) in [
            ("Write", EditorViewMode::Write),
            ("Preview", EditorViewMode::Preview),
            ("Split", EditorViewMode::Split),
        ] {
            let button = gtk::Button::with_label(label);
            if mode == EditorViewMode::Write {
                button.set_css_classes(&["nota-mode-button", "active"]);
            } else {
                button.set_css_classes(&["nota-mode-button"]);
            }
            button.set_sensitive(true);
            let mode_sender = sender.clone();
            button.connect_clicked(move |_| {
                let _send_result = mode_sender.send(AppMsg::SetViewMode(mode));
            });
            modes.append(&button);
            mode_buttons.push((mode, button));
        }
        footer_row.set_center_widget(Some(&modes));
        footer_row.set_end_widget(Some(&saved_state));
        let footer_saved_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        footer_saved_row.set_margin_top(9);
        footer_saved_row.set_visible(false);
        editor_footer.append(&footer_row);
        editor_footer.append(&footer_saved_row);
        editor.append(&editor_footer);

        let workspace = gtk::Overlay::new();
        workspace.set_vexpand(true);
        workspace.set_child(Some(&editor));
        let scrim = gtk::Button::new();
        scrim.set_css_classes(&["nota-drawer-scrim"]);
        scrim.set_focusable(false);
        scrim.set_hexpand(true);
        scrim.set_vexpand(true);
        scrim.set_visible(false);
        scrim.update_property(&[gtk::accessible::Property::Label("Close notes")]);
        workspace.add_overlay(&scrim);
        workspace.add_overlay(&sidebar);
        root.append(&workspace);
        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&root));
        overlay.add_overlay(&notification);
        window.set_child(Some(&overlay));

        for button in [&pin, &pin_menu] {
            let selected = rendered_note.clone();
            let pin_sender = sender.clone();
            let popover = note_menu.clone();
            button.connect_clicked(move |_| {
                popover.popdown();
                if let Some(id) = selected.get() {
                    let _send_result = pin_sender.send(AppMsg::TogglePin(id));
                }
            });
        }
        let selected_for_delete = rendered_note.clone();
        let delete_sender = sender.clone();
        let delete_popover = note_menu.clone();
        delete.connect_clicked(move |_| {
            delete_popover.popdown();
            if let Some(id) = selected_for_delete.get() {
                let _send_result = delete_sender.send(AppMsg::RequestDelete(id));
            }
        });
        for (popover, buttons) in [
            (&settings_menu, vec![&theme, &help]),
            (
                &backup_menu,
                vec![
                    &export_backup,
                    &import_backup,
                    &export_transition,
                    &import_transition,
                ],
            ),
            (&note_menu, vec![&edit_tags, &strike, &table]),
        ] {
            for button in buttons {
                let popover = popover.clone();
                button.connect_clicked(move |_| popover.popdown());
            }
        }

        let create_sender = sender.clone();
        create.connect_clicked(move |_| {
            let _send_result = create_sender.send(AppMsg::QuickCapture);
        });
        let edit_tags_sender = sender.clone();
        edit_tags.connect_clicked(move |_| {
            let _send_result = edit_tags_sender.send(AppMsg::StartEditTags);
        });
        let restore_sender = sender.clone();
        restore_previous.connect_clicked(move |_| {
            let _send_result = restore_sender.send(AppMsg::RestorePreviousSnapshot);
        });
        let empty_sender = sender.clone();
        start_empty.connect_clicked(move |_| {
            let _send_result = empty_sender.send(AppMsg::StartEmptyAfterRecovery);
        });
        let clear_all_sender = sender.clone();
        clear_all.connect_clicked(move |_| {
            let _send_result = clear_all_sender.send(AppMsg::RequestClearAll);
        });
        let export_backup_sender = sender.clone();
        export_backup.connect_clicked(move |_| {
            let _send_result = export_backup_sender.send(AppMsg::RequestBackupExport);
        });
        let import_backup_sender = sender.clone();
        import_backup.connect_clicked(move |_| {
            let _send_result = import_backup_sender.send(AppMsg::RequestBackupImport);
        });
        let recovery_import_sender = sender.clone();
        recovery_import.connect_clicked(move |_| {
            let _send_result = recovery_import_sender.send(AppMsg::RequestBackupImport);
        });
        let import_transition_sender = sender.clone();
        import_transition.connect_clicked(move |_| {
            let _send_result = import_transition_sender.send(AppMsg::RequestTransitionImport);
        });
        let export_transition_sender = sender.clone();
        export_transition.connect_clicked(move |_| {
            let _send_result = export_transition_sender.send(AppMsg::RequestTransitionExport);
        });
        let theme_sender = sender.clone();
        theme.connect_clicked(move |_| {
            let _send_result = theme_sender.send(AppMsg::ToggleTheme);
        });
        let diagnostics_sender = sender.clone();
        diagnostics.connect_clicked(move |_| {
            let _send_result = diagnostics_sender.send(AppMsg::RequestDiagnostics);
        });
        let help_window = window.clone();
        help.connect_clicked(move |_| {
            show_markdown_help(&help_window);
        });
        let sidebar_navigation_sender = sender.clone();
        sidebar_navigation.connect_clicked(move |_| {
            let _send_result = sidebar_navigation_sender.send(AppMsg::ToggleNavigation);
        });
        let scrim_sender = sender.clone();
        scrim.connect_clicked(move |_| {
            let _send_result = scrim_sender.send(AppMsg::ToggleNavigation);
        });
        let editor_navigation_sender = sender.clone();
        editor_navigation.connect_clicked(move |_| {
            let _send_result = editor_navigation_sender.send(AppMsg::ToggleNavigation);
        });
        let refreshing = Rc::new(Cell::new(false));
        let search_sender = sender.clone();
        let search_refreshing = refreshing.clone();
        search.connect_changed(move |entry| {
            if !search_refreshing.get() {
                let _send_result = search_sender.send(AppMsg::EditSearch(entry.text().to_string()));
            }
        });
        let sidebar_for_stop_search = sidebar.downgrade();
        let stop_search_sender = sender.clone();
        search.connect_stop_search(move |_| {
            if sidebar_for_stop_search
                .upgrade()
                .is_some_and(|sidebar| sidebar.is_visible())
            {
                let _send_result = stop_search_sender.send(AppMsg::ToggleNavigation);
            }
        });
        let hint = search_hint.clone();
        let focus = gtk::EventControllerFocus::new();
        focus.connect_enter(move |_| {
            hint.popup();
        });
        let hint = search_hint.clone();
        focus.connect_leave(move |_| {
            hint.popdown();
        });
        search.add_controller(focus);
        let filter_sender = sender.clone();
        filter_chip.connect_clicked(move |_| {
            let _send_result = filter_sender.send(AppMsg::ClearTag);
        });
        let title_refreshing = refreshing.clone();
        let title_sender = sender.clone();
        title.connect_changed(move |entry| {
            // GTK4 Entry puts keyboard focus on an inner GtkText, so has_focus() is
            // false while typing. FOCUS_WITHIN lets title edits reach the model.
            if !title_refreshing.get() && entry_has_input_focus(entry) {
                let _send_result = title_sender.send(AppMsg::UpdateTitle(entry.text().to_string()));
            }
        });
        let tags_refreshing = refreshing.clone();
        let tags_sender = sender.clone();
        tags.connect_changed(move |entry| {
            if !tags_refreshing.get() && entry_has_input_focus(entry) {
                let _send_result = tags_sender.send(AppMsg::UpdateTags(entry.text().to_string()));
            }
        });
        let finish_tags_sender = sender.clone();
        let tags_focus = gtk::EventControllerFocus::new();
        tags_focus.connect_leave(move |_| {
            let _send_result = finish_tags_sender.send(AppMsg::FinishEditTags);
        });
        tags.add_controller(tags_focus);
        let tag_suggestion_count = Rc::new(Cell::new(0));
        let tags_key = gtk::EventControllerKey::new();
        let tags_for_key = tags.clone();
        let accept_tag_sender = sender.clone();
        let tag_suggestion_count_for_key = tag_suggestion_count.clone();
        tags_key.connect_key_pressed(move |_, keyval, _, _| {
            if tag_suggestion_count_for_key.get() == 0 {
                return gtk::glib::Propagation::Proceed;
            }
            if keyval == gtk::gdk::Key::Return || keyval == gtk::gdk::Key::Tab {
                let _send_result = accept_tag_sender
                    .send(AppMsg::AcceptTagSuggestion(tags_for_key.text().to_string()));
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        tags.add_controller(tags_key);
        let content_refreshing = refreshing.clone();
        let content_sender = sender.clone();
        let content_view = content.clone();
        let content_changed = Rc::new(content.buffer().connect_changed(move |buffer| {
            if !content_refreshing.get() && content_view.has_focus() {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let _send_result = content_sender.send(AppMsg::UpdateContent(text.to_string()));
            }
        }));
        connect_formatting_button(
            &bold,
            &content,
            &writing,
            MarkdownCommand::Bold,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &italic,
            &content,
            &writing,
            MarkdownCommand::Italic,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &heading,
            &content,
            &writing,
            MarkdownCommand::Heading,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &bullet_list,
            &content,
            &writing,
            MarkdownCommand::BulletList,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &link,
            &content,
            &writing,
            MarkdownCommand::Link,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &strike,
            &content,
            &writing,
            MarkdownCommand::Strikethrough,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &task,
            &content,
            &writing,
            MarkdownCommand::TaskList,
            sender,
            &content_changed,
        );
        connect_formatting_button(
            &table,
            &content,
            &writing,
            MarkdownCommand::Table,
            sender,
            &content_changed,
        );

        let shortcuts = gtk::ShortcutController::new();
        let quick_capture_sender = sender.clone();
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<Control>n"),
            Some(gtk::CallbackAction::new(move |_, _| {
                let _send_result = quick_capture_sender.send(AppMsg::QuickCapture);
                gtk::glib::Propagation::Stop
            })),
        ));
        let search_for_shortcut = search.clone();
        let sidebar_for_search = sidebar.clone();
        let search_shortcut_sender = sender.clone();
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<Control>f"),
            Some(gtk::CallbackAction::new(move |_, _| {
                if !sidebar_for_search.is_visible() {
                    let _send_result = search_shortcut_sender.send(AppMsg::ToggleNavigation);
                }
                let search = search_for_shortcut.clone();
                gtk::glib::idle_add_local_once(move || {
                    search.grab_focus();
                });
                gtk::glib::Propagation::Stop
            })),
        ));
        let sidebar_for_escape = sidebar.clone();
        let escape_sender = sender.clone();
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("Escape"),
            Some(gtk::CallbackAction::new(move |_, _| {
                if sidebar_for_escape.is_visible() {
                    let _send_result = escape_sender.send(AppMsg::ToggleNavigation);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            })),
        ));
        window.add_controller(shortcuts);

        let resize_sender = sender.clone();
        window.connect_realize(move |window| {
            let Some(surface) = window.surface() else {
                return;
            };
            let window = window.downgrade();
            let resize_sender = resize_sender.clone();
            let previous_width = Rc::new(Cell::new(0));
            surface.connect_layout(move |_, _, _| {
                let window = window.clone();
                let resize_sender = resize_sender.clone();
                let previous_width = previous_width.clone();
                gtk::glib::idle_add_local_once(move || {
                    if let Some(window) = window.upgrade() {
                        let width = window.width();
                        if width > 0 && previous_width.replace(width) != width {
                            let _send_result = resize_sender.send(AppMsg::Resize(width as f64));
                        }
                    }
                });
            });
        });

        install_css();
        Self {
            root,
            sidebar,
            editor,
            editor_header,
            editor_tools,
            editor_empty,
            formatting,
            footer_row,
            footer_saved_row,
            saved_state,
            modes,
            editor_navigation,
            scrim,
            previous_focus: gtk::glib::WeakRef::new(),
            deleted_count,
            metadata,
            pin,
            pin_menu,
            note_actions,
            menus: vec![settings_menu, backup_menu, note_menu],
            all_filter,
            pinned_filter,
            title,
            tags,
            content,
            status,
            statistics,
            notes_count,
            empty_state,
            notification,
            recovery_panel,
            recovery_actions,
            restore_previous,
            create,
            search,
            backup_dot,
            backup_label,
            tag_suggestions,
            edit_tags,
            cleanup_tags,
            filter_chip,
            clear_all,
            theme_label,
            writing,
            content_scroll,
            surface_scroll,
            surface_row,
            #[cfg(feature = "preview-webkit")]
            preview,
            #[cfg(not(feature = "preview-webkit"))]
            preview_fallback,
            tag_suggestion_count,
            mode_buttons,
            refreshing,
            rendered_note,
        }
    }

    pub(super) fn title_input(&self) -> gtk::Entry {
        self.title.clone()
    }
    pub(super) fn tags_input(&self) -> gtk::Entry {
        self.tags.clone()
    }

    pub(super) fn refresh(
        &self,
        app: &AppModel,
        window: &gtk::ApplicationWindow,
        recovery: Option<&NativeRecovery>,
        sender: &relm4::Sender<AppMsg>,
    ) {
        self.refreshing.set(true);
        let selected = app.workspace.selected_id();
        let selection_changed = self.rendered_note.replace(selected) != selected;
        let dark = matches!(app.theme, nota_core::transition::ThemePreference::Dark);
        if dark {
            self.root.add_css_class("nota-dark");
            window.add_css_class("nota-dark");
        } else {
            self.root.remove_css_class("nota-dark");
            window.remove_css_class("nota-dark");
        }
        self.theme_label
            .set_label(if dark { "Light Theme" } else { "Dark Theme" });
        let full_width_drawer = window.width() > 0 && window.width() <= 560;
        self.recovery_actions.set_orientation(if full_width_drawer {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        });
        let split = app.view_mode == EditorViewMode::Split;
        self.root.set_class_active("nota-split", split);
        self.root
            .set_class_active("nota-medium", window.width() <= 760);
        self.root
            .set_class_active("nota-compact", full_width_drawer);
        window.set_class_active("nota-compact", full_width_drawer);
        if full_width_drawer != self.footer_saved_row.is_visible() {
            self.footer_row.set_end_widget(None::<&gtk::Widget>);
            if full_width_drawer {
                self.footer_row.set_center_widget(None::<&gtk::Widget>);
                self.footer_row.set_end_widget(Some(&self.modes));
                self.footer_saved_row.append(&self.saved_state);
            } else {
                self.footer_saved_row.remove(&self.saved_state);
                self.footer_row.set_center_widget(Some(&self.modes));
                self.footer_row.set_end_widget(Some(&self.saved_state));
            }
            self.footer_saved_row.set_visible(full_width_drawer);
        }
        if self.search.text().as_str() != app.note_list.search_input() {
            self.search.set_text(app.note_list.search_input());
        }
        if let Some(body_plane) = self.content.parent() {
            body_plane.set_margin_top(if full_width_drawer { 28 } else { 31 });
            let inset = if full_width_drawer {
                24
            } else if split {
                27
            } else {
                42
            };
            body_plane.set_margin_start(inset);
            body_plane.set_margin_end(if split && !full_width_drawer {
                14
            } else {
                inset
            });
        }
        let drawer_was_open = self.sidebar.is_visible();
        if app.note_list_visible && !drawer_was_open {
            self.previous_focus
                .set(gtk::prelude::GtkWindowExt::focus(window).as_ref());
        }
        self.sidebar.set_visible(app.note_list_visible);
        self.sidebar.set_halign(if full_width_drawer {
            gtk::Align::Fill
        } else {
            gtk::Align::Start
        });
        self.scrim.set_visible(app.note_list_visible);
        self.editor.set_sensitive(!app.note_list_visible);
        self.editor_navigation
            .update_state(&[gtk::accessible::State::Expanded(Some(
                app.note_list_visible,
            ))]);
        self.editor_navigation
            .set_class_active("active", app.note_list_visible);
        if drawer_was_open && !app.note_list_visible {
            if let Some(previous) = self
                .previous_focus
                .upgrade()
                .filter(|widget| widget.is_sensitive() && widget.is_visible())
            {
                previous.grab_focus();
            } else {
                self.title.grab_focus();
            }
        }
        for menu in &self.menus {
            menu.set_class_active("nota-dark", dark);
            if let Some(child) = menu.child() {
                child.set_class_active("nota-dark", dark);
            }
        }
        self.all_filter
            .set_class_active("active", !app.note_list.pinned_only());
        self.pinned_filter
            .set_class_active("active", app.note_list.pinned_only());
        self.deleted_count
            .set_text(&app.workspace.recently_deleted_notes().len().to_string());
        let backup_health = app.backup_health_status(chrono::Utc::now());
        let backup_dot_class = match backup_health {
            BackupHealth::Missing => "missing",
            BackupHealth::Recent { .. } => "recent",
            BackupHealth::Stale { .. } => "stale",
        };
        self.backup_dot
            .set_css_classes(&["nota-backup-dot", backup_dot_class]);
        self.backup_label
            .set_label(app.backup_health_label(chrono::Utc::now()));
        self.recovery_panel.set_visible(recovery.is_some());
        let recovering = recovery.is_some() || app.is_in_storage_recovery();
        let has_note = selected.is_some() && !recovering;
        self.editor_header.set_visible(has_note);
        self.editor_tools.set_visible(has_note);
        self.editor_empty
            .set_visible(selected.is_none() && !recovering);
        self.surface_row.set_visible(has_note);
        self.pin.set_sensitive(!recovering && selected.is_some());
        self.note_actions
            .set_sensitive(!recovering && selected.is_some());
        self.create.set_sensitive(!recovering);
        self.search.set_sensitive(!recovering);
        self.restore_previous.set_sensitive(
            recovery
                .as_ref()
                .and_then(|recovery| recovery.previous_snapshot.as_ref())
                .is_some(),
        );
        self.clear_all
            .set_visible(!app.workspace.recently_deleted_notes().is_empty());
        let list = app.note_list_render_model();
        self.cleanup_tags.set_visible(
            !recovering && app.is_editing_tags() && !app.workspace.tag_cleanup_plan().is_empty(),
        );
        let rendered_notes = list.projection.rows.len();
        self.notes_count.set_text(&rendered_notes.to_string());
        match list.display_state {
            NoteListDisplayState::EmptyCollection => {
                self.empty_state.set_visible(true);
                self.empty_state.set_text("No notes yet.");
            }
            NoteListDisplayState::FilteredEmpty => {
                self.empty_state.set_visible(true);
                self.empty_state.set_text("No matching notes.");
            }
            NoteListDisplayState::Rows => self.empty_state.set_visible(false),
        }
        if let Some(tag) = app.note_list.active_tag() {
            self.filter_chip.set_label(&format!("#{tag} ×"));
            self.filter_chip
                .set_tooltip_text(Some(&format!("Clear {tag} filter")));
            self.filter_chip.set_visible(true);
        } else {
            self.filter_chip.set_visible(false);
        }
        let editing_tags = app.is_editing_tags();
        self.tags.set_visible(editing_tags);
        self.edit_tags.set_visible(!recovering && !editing_tags);
        self.metadata.set_visible(!editing_tags);
        if recovering {
            self.title.set_sensitive(false);
            self.tags.set_sensitive(false);
            self.content.set_sensitive(false);
            self.edit_tags.set_sensitive(false);
        } else if let Some(note) = app.workspace.selected_note() {
            if (selection_changed || !entry_has_input_focus(&self.title))
                && self.title.text().as_str() != note.title
            {
                self.title.set_text(&note.title);
            }
            let tags = note.tags.join(", ");
            if (selection_changed || !entry_has_input_focus(&self.tags))
                && self.tags.text().as_str() != tags
            {
                self.tags.set_text(&tags);
            }
            let age = chrono::Utc::now().signed_duration_since(note.last_modified);
            let edited = if age.num_minutes() < 1 {
                "Edited just now".to_string()
            } else if age.num_hours() < 1 {
                format!("Edited {} min ago", age.num_minutes())
            } else {
                format!("Edited {}", note.last_modified.format("%d %b"))
            };
            let mut metadata = note
                .tags
                .iter()
                .map(|tag| {
                    let query = url::form_urlencoded::Serializer::new(String::new())
                        .append_pair("tag", tag)
                        .finish();
                    format!(
                        "<a href=\"nota:tag?{}\">#{}</a>",
                        gtk::glib::markup_escape_text(&query),
                        gtk::glib::markup_escape_text(tag)
                    )
                })
                .collect::<Vec<_>>()
                .join("   ");
            if !metadata.is_empty() {
                metadata.push_str("   ");
            }
            metadata.push_str("<a href=\"nota:edit-tags\">+</a>   ·   ");
            metadata.push_str(&edited);
            self.metadata.set_markup(&metadata);
            self.pin.set_class_active("pinned", note.is_pinned);
            self.pin.set_tooltip_text(Some(if note.is_pinned {
                "Unpin note"
            } else {
                "Pin note"
            }));
            self.pin_menu.set_label(if note.is_pinned {
                "Unpin note"
            } else {
                "Pin note"
            });
            self.edit_tags.set_sensitive(true);
            let buffer = self.content.buffer();
            let current = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            if selection_changed || (!self.content.has_focus() && current.as_str() != note.content)
            {
                buffer.set_text(&note.content);
            }
            self.title.set_sensitive(true);
            self.tags.set_sensitive(true);
            self.content.set_sensitive(true);
            let words = note.word_count();
            self.statistics.set_text(&format!(
                "{} {}",
                words,
                if words == 1 { "word" } else { "words" }
            ));
        } else {
            self.title.set_text("");
            self.tags.set_text("");
            self.content.buffer().set_text("");
            self.title.set_sensitive(false);
            self.tags.set_sensitive(false);
            self.content.set_sensitive(false);
            self.statistics.set_text("0 words");
            self.metadata.set_text("");
            self.edit_tags.set_visible(false);
        }
        while let Some(child) = self.tag_suggestions.first_child() {
            self.tag_suggestions.remove(&child);
        }
        if editing_tags && !recovering {
            let suggestions = app.tag_suggestions(self.tags.text().as_str());
            self.tag_suggestion_count.set(suggestions.len());
            for suggestion in suggestions {
                let completed = suggestion.completed_input.clone();
                let label = gtk::Label::new(Some(&format!("#{}", suggestion.label)));
                label.set_halign(gtk::Align::Start);
                label.set_xalign(0.0);
                label.set_hexpand(true);
                let button = gtk::Button::new();
                button.set_child(Some(&label));
                button.set_halign(gtk::Align::Fill);
                button.set_valign(gtk::Align::Start);
                button.set_focus_on_click(false);
                button.set_css_classes(&["nota-tag-suggestion"]);
                let tags_entry = self.tags.clone();
                let suggestion_sender = sender.clone();
                button.connect_clicked(move |_| {
                    tags_entry.set_text(&completed);
                    let _send_result =
                        suggestion_sender.send(AppMsg::UpdateTags(completed.clone()));
                });
                self.tag_suggestions.append(&button);
            }
            self.tag_suggestions
                .set_visible(self.tag_suggestion_count.get() > 0);
        } else {
            self.tag_suggestion_count.set(0);
            self.tag_suggestions.set_visible(false);
        }
        self.status.set_text(match app.save_status {
            SaveStatus::Saved => "Saved locally",
            SaveStatus::Saving => "Saving…",
            SaveStatus::Failed => "Save failed",
        });
        if let Some(notification) = &app.notification {
            self.notification.set_text(&notification.message);
            self.notification.set_css_classes(&[
                "nota-notification",
                match notification.tone {
                    NotificationTone::Progress => "nota-notification-progress",
                    NotificationTone::Success => "nota-notification-success",
                    NotificationTone::Error => "nota-notification-error",
                },
            ]);
            self.notification.set_visible(true);
        } else {
            self.notification.set_visible(false);
        }
        let surfaces = app.view_mode.surfaces();
        let stacked = split && full_width_drawer;
        self.surface_scroll.set_vscrollbar_policy(if stacked {
            gtk::PolicyType::Automatic
        } else {
            gtk::PolicyType::Never
        });
        self.content_scroll.set_propagate_natural_height(stacked);
        self.content_scroll.set_vscrollbar_policy(if stacked {
            gtk::PolicyType::Never
        } else {
            gtk::PolicyType::Automatic
        });
        self.content.set_bottom_margin(0);
        if let Some(plane) = self.content.parent() {
            plane.set_margin_bottom(if stacked { 0 } else { 45 });
        }
        self.content
            .set_height_request(if stacked { 340 } else { -1 });
        self.writing.set_vexpand(!stacked);
        self.formatting.set_visible(has_note && surfaces.writing);
        self.surface_row
            .set_orientation(if split && full_width_drawer {
                gtk::Orientation::Vertical
            } else {
                gtk::Orientation::Horizontal
            });
        self.surface_row
            .set_homogeneous(surfaces.writing && surfaces.preview && !stacked);
        self.writing.set_visible(surfaces.writing);
        for (mode, button) in &self.mode_buttons {
            if *mode == app.view_mode {
                button.set_css_classes(&["nota-mode-button", "active"]);
            } else {
                button.set_css_classes(&["nota-mode-button"]);
            }
            button.set_sensitive(has_note);
        }
        #[cfg(feature = "preview-webkit")]
        {
            let layout = if !split {
                nota_app::preview::PreviewLayout::Reading
            } else if window.width() <= 760 {
                nota_app::preview::PreviewLayout::SplitNarrow
            } else {
                nota_app::preview::PreviewLayout::Split
            };
            self.preview.widget().set_visible(surfaces.preview);
            self.preview
                .widget()
                .set_height_request(if stacked { 500 } else { -1 });
            if let Some(plane) = self.preview.widget().parent() {
                plane.set_visible(surfaces.preview);
                plane.set_margin_top(if stacked {
                    18
                } else if full_width_drawer {
                    28
                } else {
                    31
                });
                plane.set_class_active("nota-split-preview", split);
                plane.set_margin_start(if full_width_drawer {
                    24
                } else if split {
                    14
                } else {
                    42
                });
                plane.set_margin_end(if full_width_drawer {
                    24
                } else if split {
                    27
                } else {
                    42
                });
            }
            if let Some(note) = app.workspace.selected_note() {
                self.preview.load_note(
                    note.display_title(),
                    &note.content,
                    matches!(app.theme, nota_core::transition::ThemePreference::Dark),
                    layout,
                );
            } else {
                self.preview.load_note("", "", dark, layout);
            }
        }
        #[cfg(not(feature = "preview-webkit"))]
        {
            self.preview_fallback.set_visible(surfaces.preview);
        }
        self.refreshing.set(false);
    }
}

fn icon_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon);
    button.set_css_classes(&["nota-icon-button"]);
    button.set_tooltip_text(Some(label));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

fn icon_label_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::new();
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    row.append(&gtk::Image::from_icon_name(icon));
    row.append(&gtk::Label::new(Some(label)));
    button.set_child(Some(&row));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

/// GTK4 `Entry` keeps keyboard focus on an inner `GtkText`, so `has_focus()` is
/// false while the user is editing. `FOCUS_WITHIN` is set on the Entry itself.
fn entry_has_input_focus(entry: &gtk::Entry) -> bool {
    entry.has_focus() || entry.state_flags().contains(gtk::StateFlags::FOCUS_WITHIN)
}

fn formatting_button(label: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.set_css_classes(&["nota-toolbar-button"]);
    button.set_tooltip_text(Some(tooltip));
    button
}

fn connect_formatting_button(
    button: &gtk::Button,
    content: &gtk::TextView,
    writing: &gtk::Box,
    command: MarkdownCommand,
    sender: &relm4::Sender<AppMsg>,
    content_changed: &Rc<gtk::glib::SignalHandlerId>,
) {
    let content = content.clone();
    let writing = writing.clone();
    let sender = sender.clone();
    let content_changed = content_changed.clone();
    button.set_focus_on_click(false);
    button.connect_clicked(move |_| {
        let buffer = content.buffer();
        let (mut start, mut end) = buffer.selection_bounds().unwrap_or_else(|| {
            let cursor = buffer.iter_at_offset(buffer.cursor_position());
            (cursor, cursor)
        });
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
        let Some(selection) = gtk_character_range_to_byte_selection(
            &text,
            start.offset().max(0) as usize,
            end.offset().max(0) as usize,
        ) else {
            return;
        };
        let (selection_start_byte, _) = selection.ordered();
        let formatted = apply_markdown_command(&text, selection, command);
        let replacement = &formatted.content[selection_start_byte..formatted.caret_byte];
        buffer.block_signal(&content_changed);
        buffer.begin_user_action();
        buffer.delete(&mut start, &mut end);
        buffer.insert(&mut start, replacement);
        let caret = formatted.content[..formatted.caret_byte].chars().count() as i32;
        buffer.place_cursor(&buffer.iter_at_offset(caret));
        buffer.end_user_action();
        buffer.unblock_signal(&content_changed);
        let _send_result = sender.send(AppMsg::UpdateContent(formatted.content));
        if !writing.is_visible() {
            let _send_result = sender.send(AppMsg::SetViewMode(EditorViewMode::Write));
        }
        let content_for_focus = content.clone();
        gtk::glib::idle_add_local_once(move || {
            content_for_focus.grab_focus();
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    #[ignore = "requires a GTK display; run mise run test:gtk"]
    fn gtk_toolbar_formatting_preserves_unicode_caret_and_undo() {
        gtk::init().expect("the GTK workflow test requires a display");
        let view = gtk::TextView::new();
        let buffer = view.buffer();
        buffer.set_text("A😀B");
        buffer.select_range(&buffer.iter_at_offset(1), &buffer.iter_at_offset(2));
        let changed = Rc::new(buffer.connect_changed(|_| {}));
        let (sender, _receiver) = relm4::channel();
        let bold = formatting_button("B", "Bold");
        let writing = gtk::Box::new(gtk::Orientation::Vertical, 0);
        connect_formatting_button(
            &bold,
            &view,
            &writing,
            MarkdownCommand::Bold,
            &sender,
            &changed,
        );
        bold.emit_clicked();
        let text = || {
            buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string()
        };
        assert_eq!(text(), "A**😀**B");
        assert_eq!(buffer.cursor_position(), 6);
        assert!(buffer.can_undo());
        buffer.undo();
        assert_eq!(text(), "A😀B");
        assert!(buffer.can_redo());
        buffer.redo();
        assert_eq!(text(), "A**😀**B");
        buffer.insert_at_cursor("!");
        assert_eq!(text(), "A**😀**!B");
    }

    #[test]
    #[ignore = "requires a GTK display"]
    fn gtk_focus_reading_plane_centers_and_shrinks_without_css_errors() {
        gtk::init().expect("the GTK layout test requires a display");
        let errors = Rc::new(RefCell::new(Vec::new()));
        let errors_for_signal = errors.clone();
        let provider = gtk::CssProvider::new();
        provider.connect_parsing_error(move |_, _, error| {
            errors_for_signal.borrow_mut().push(error.to_string());
        });
        provider.load_from_string(nota_desktop::visual_contract::NATIVE_STYLESHEET);
        assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());

        let plane = WritingPlane::new(NATIVE_VISUAL_CONTRACT.editor_measure_px);
        let body = gtk::TextView::new();
        plane.set_child(Some(&body));
        plane.allocate(900, 100, -1, None);
        let bounds = body.compute_bounds(&plane).expect("reading plane bounds");
        assert_eq!(bounds.x(), 150.0);
        assert_eq!(bounds.width(), 600.0);
        plane.allocate(400, 100, -1, None);
        let bounds = body
            .compute_bounds(&plane)
            .expect("compact reading plane bounds");
        assert_eq!(bounds.x(), 0.0);
        assert_eq!(bounds.width(), 400.0);
    }
}
