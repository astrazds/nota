use super::dialogs::show_markdown_help;
use super::note_list::NoteLists;
use super::style::{install_css, install_workspace_fonts, measure_ch_width_px};
use super::writing_plane::WritingPlane;
use nota_core::backup::BackupHealth;
use nota_core::editor_view::EditorViewMode;
use nota_core::markdown_editing::MarkdownCommand;
use nota_core::note_list_interaction::NoteListDisplayState;
use nota_desktop::app::{AppModel, AppMsg, NotificationTone, SaveStatus};
use nota_desktop::selection::gtk_character_range_to_byte_selection;
use nota_desktop::storage::NativeRecovery;
use nota_desktop::visual_contract::{NATIVE_VISUAL_CONTRACT, writing_plane_max_width_px};
#[cfg(feature = "preview-webkit")]
use nota_desktop::webkit_preview::SecurePreview;
use relm4::gtk;
use relm4::gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub(super) struct DesktopWidgets {
    root: gtk::Box,
    sidebar: gtk::Box,
    editor: gtk::Box,
    sidebar_navigation: gtk::Button,
    editor_navigation: gtk::Button,
    divider: gtk::Separator,
    title: gtk::Entry,
    tags: gtk::Entry,
    content: gtk::TextView,
    status: gtk::Label,
    statistics: gtk::Label,
    notes_count: gtk::Label,
    empty_state: gtk::Box,
    notification: gtk::Label,
    recovery_panel: gtk::Box,
    restore_previous: gtk::Button,
    create: gtk::Button,
    search: gtk::SearchEntry,
    backup_dot: gtk::Label,
    backup_label: gtk::Label,
    result_status: gtk::Label,
    empty_title: gtk::Label,
    empty_copy: gtk::Label,
    empty_create: gtk::Button,
    tags_pills: gtk::Box,
    tag_suggestions: gtk::Box,
    edit_tags: gtk::Button,
    filter_row: gtk::Box,
    filter_chip: gtk::Button,
    clear_all: gtk::Button,
    theme_label: gtk::Label,
    writing: gtk::Box,
    surface_row: gtk::Box,
    #[cfg(feature = "preview-webkit")]
    preview: SecurePreview,
    /// Non-webkit Preview/Split surface (message stub).
    #[cfg(not(feature = "preview-webkit"))]
    preview_fallback: gtk::Box,
    tag_suggestion_count: Rc<Cell<usize>>,
    mode_buttons: Vec<(EditorViewMode, gtk::Button)>,
}

impl DesktopWidgets {
    pub(super) fn new(
        window: &gtk::ApplicationWindow,
        note_lists: &NoteLists,
        recovery: Option<&NativeRecovery>,
        sender: &relm4::Sender<AppMsg>,
    ) -> Self {
        install_workspace_fonts(window);
        let writing_plane_max_px = writing_plane_max_width_px(measure_ch_width_px(window));
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.set_css_classes(&["nota-root"]);

        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar.set_width_request(NATIVE_VISUAL_CONTRACT.sidebar_width);
        sidebar.set_css_classes(&["nota-sidebar"]);

        let sidebar_header = gtk::Box::new(gtk::Orientation::Vertical, 12);
        sidebar_header.set_css_classes(&["nota-sidebar-header"]);
        let identity = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let app_title = gtk::Label::new(Some("Nota"));
        app_title.set_hexpand(true);
        app_title.set_halign(gtk::Align::Start);
        app_title.set_css_classes(&["nota-app-title"]);
        let sidebar_navigation = gtk::Button::with_label("Writing");
        sidebar_navigation.set_tooltip_text(Some("Close the Note List"));
        sidebar_navigation.set_css_classes(&["nota-navigation-button"]);
        identity.append(&app_title);
        identity.append(&sidebar_navigation);
        sidebar_header.append(&identity);

        let commands = gtk::Box::new(gtk::Orientation::Vertical, 2);
        commands.set_css_classes(&["nota-command-list"]);
        let create = command_button("＋", "New Note", "Ctrl N");
        create.add_css_class("nota-command-primary");
        create.set_accessible_role(gtk::AccessibleRole::Button);
        let (theme, theme_label) = command_button_parts("◐", "Dark Theme", "");
        theme.set_tooltip_text(Some("Toggle Light and Dark Theme"));
        let focus_search = command_button("⌕", "Search", "Ctrl F");
        let diagnostics = command_button("ⓘ", "About Nota", "");
        diagnostics.set_tooltip_text(Some("Show version, storage, and Backup Health"));
        commands.append(&create);
        commands.append(&theme);
        commands.append(&focus_search);
        commands.append(&diagnostics);
        sidebar_header.append(&commands);

        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search Notes, #tags, title:…")
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
        let filter_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        filter_row.set_css_classes(&["nota-filter-row"]);
        let filter_prefix = gtk::Label::new(Some("Filtered by"));
        filter_prefix.set_css_classes(&["nota-footer-label"]);
        let filter_chip = gtk::Button::with_label("#tag");
        filter_chip.set_tooltip_text(Some("Clear tag filter"));
        filter_chip.set_css_classes(&["nota-footer-button"]);
        filter_row.append(&filter_prefix);
        filter_row.append(&filter_chip);
        filter_row.set_visible(false);
        sidebar_header.append(&search);
        sidebar_header.append(&filter_row);
        sidebar.append(&sidebar_header);

        let sidebar_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();
        sidebar_scroll.set_css_classes(&["nota-sidebar-scroll"]);
        let sidebar_content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let notes_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        notes_header.set_css_classes(&["nota-section-header"]);
        let notes_label = gtk::Label::new(Some("Notes"));
        notes_label.set_hexpand(true);
        notes_label.set_halign(gtk::Align::Start);
        let notes_count = gtk::Label::new(None);
        notes_count.set_css_classes(&["nota-section-count"]);
        notes_header.append(&notes_label);
        notes_header.append(&notes_count);
        sidebar_content.append(&notes_header);
        let result_status = gtk::Label::new(None);
        result_status.set_halign(gtk::Align::Start);
        result_status.set_wrap(true);
        result_status.set_css_classes(&["nota-result-status"]);
        result_status.set_visible(false);
        sidebar_content.append(&result_status);
        let empty_state = gtk::Box::new(gtk::Orientation::Vertical, 4);
        empty_state.set_css_classes(&["nota-empty-state"]);
        let empty_title = gtk::Label::new(Some("A quiet place for your notes"));
        empty_title.set_wrap(true);
        empty_title.set_css_classes(&["nota-empty-title"]);
        let empty_copy = gtk::Label::new(Some("Create a Note to start writing."));
        empty_copy.set_wrap(true);
        empty_copy.set_css_classes(&["nota-empty-copy"]);
        empty_state.append(&empty_title);
        empty_state.append(&empty_copy);
        let empty_create = gtk::Button::with_label("Create a Note");
        empty_create.set_css_classes(&["nota-footer-button"]);
        empty_state.append(&empty_create);
        sidebar_content.append(&empty_state);
        sidebar_content.append(note_lists.notes_widget());

        let data_actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        data_actions.set_hexpand(true);
        data_actions.set_halign(gtk::Align::End);
        let export_backup = gtk::Button::with_label("Export");
        export_backup.set_tooltip_text(Some("Export merge Backup"));
        let import_backup = gtk::Button::with_label("Import");
        import_backup.set_tooltip_text(Some("Import merge Backup"));
        let import_transition = gtk::Button::with_label("Restore");
        import_transition.set_tooltip_text(Some(
            "Restore a desktop transition into an Empty Collection",
        ));
        for button in [&export_backup, &import_backup, &import_transition] {
            button.set_css_classes(&["nota-footer-button"]);
        }
        data_actions.append(&export_backup);
        data_actions.append(&import_backup);
        data_actions.append(&import_transition);

        let deleted_label = gtk::Label::new(Some("Recently Deleted"));
        deleted_label.set_halign(gtk::Align::Start);
        deleted_label.set_hexpand(true);
        let deleted_header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        deleted_header.set_css_classes(&["nota-deleted-header"]);
        deleted_header.append(&deleted_label);
        let clear_all = gtk::Button::with_label("Clear All");
        clear_all.set_halign(gtk::Align::End);
        clear_all.set_css_classes(&["nota-small-button", "danger"]);
        deleted_header.append(&clear_all);
        let deleted_panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        deleted_panel.set_css_classes(&["nota-deleted-panel"]);
        deleted_panel.append(&deleted_header);
        deleted_panel.append(note_lists.deleted_widget());
        sidebar_content.append(&deleted_panel);
        sidebar_scroll.set_child(Some(&sidebar_content));
        sidebar.append(&sidebar_scroll);

        let sidebar_footer = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        sidebar_footer.set_height_request(NATIVE_VISUAL_CONTRACT.footer_height);
        sidebar_footer.set_css_classes(&["nota-sidebar-footer"]);
        let backup_status = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let backup_dot = gtk::Label::new(Some("●"));
        backup_dot.set_css_classes(&["nota-backup-dot"]);
        let backup_label = gtk::Label::new(Some("Backup"));
        backup_label.set_css_classes(&["nota-footer-label"]);
        backup_status.append(&backup_dot);
        backup_status.append(&backup_label);
        sidebar_footer.append(&backup_status);
        sidebar_footer.append(&data_actions);
        sidebar.append(&sidebar_footer);

        let editor = gtk::Box::new(gtk::Orientation::Vertical, 0);
        editor.set_hexpand(true);
        editor.set_css_classes(&["nota-editor"]);

        let editor_navigation = gtk::Button::with_label("Notes");
        editor_navigation.set_halign(gtk::Align::Start);
        editor_navigation.set_tooltip_text(Some("Open the Note List"));
        editor_navigation.set_css_classes(&["nota-navigation-button"]);

        let editor_header = gtk::Box::new(gtk::Orientation::Vertical, 4);
        editor_header.set_css_classes(&["nota-editor-header"]);
        editor_header.append(&editor_navigation);

        let title = gtk::Entry::builder()
            .placeholder_text("Note Title")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        title.set_css_classes(&["nota-title"]);
        title.set_hexpand(true);
        title.set_halign(gtk::Align::Fill);
        // Single-line title: grow horizontally within the plane (no wrap/ellipsis).
        title.set_truncate_multiline(true);
        let tags = gtk::Entry::builder()
            .placeholder_text("Add tags, separated by commas")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        tags.set_css_classes(&["nota-tags"]);
        tags.set_hexpand(true);
        tags.set_halign(gtk::Align::Fill);
        // Left-aligned 72ch writing plane for title + tags (web note_measure parity).
        let header_inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
        header_inner.set_hexpand(true);
        header_inner.set_halign(gtk::Align::Fill);
        header_inner.append(&title);
        let tags_pills = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        tags_pills.set_css_classes(&["nota-tag-pills"]);
        let edit_tags = gtk::Button::with_label("Edit tags");
        edit_tags.set_css_classes(&["nota-footer-button"]);
        header_inner.append(&tags_pills);
        header_inner.append(&edit_tags);
        header_inner.append(&tags);
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

        let content = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::WordChar)
            .hexpand(true)
            .vexpand(true)
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        content.set_css_classes(&["nota-writing-surface"]);
        content.set_left_margin(0);
        content.set_right_margin(0);
        content.set_top_margin(20);
        content.set_bottom_margin(32);

        let formatting = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        formatting.set_css_classes(&["nota-formatting-toolbar"]);
        let formatting_inner = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let bold = formatting_button("B", "Bold");
        let italic = formatting_button("I", "Italic");
        let strike = formatting_button("S", "Strikethrough");
        let task = formatting_button("☐", "Task list");
        let table = formatting_button("▦", "Table");
        formatting_inner.append(&bold);
        formatting_inner.append(&italic);
        formatting_inner.append(&strike);
        formatting_inner.append(&task);
        formatting_inner.append(&table);
        let formatting_row = WritingPlane::new(writing_plane_max_px);
        formatting_row.set_child(Some(&formatting_inner));
        formatting.append(&formatting_row);

        let status = gtk::Label::new(Some("Saved"));
        status.set_halign(gtk::Align::End);
        status.set_css_classes(&["nota-save-status"]);
        let statistics = gtk::Label::new(Some("0 lines · 0 words · 0 chars"));
        statistics.set_hexpand(true);
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
        writing.append(&editor_header);
        writing.append(&formatting);
        // Left-aligned 72ch body plane inside the scroll viewport.
        let body_plane = WritingPlane::new(writing_plane_max_px);
        body_plane.set_margin_start(32);
        body_plane.set_margin_end(32);
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
        editor.append(&surface_row);
        surface_row.append(&writing);
        #[cfg(feature = "preview-webkit")]
        let preview = {
            let preview = SecurePreview::new(|target| {
                if let Err(error) = gtk::gio::AppInfo::launch_default_for_uri(
                    target.as_str(),
                    None::<&gtk::gio::AppLaunchContext>,
                ) {
                    eprintln!("Nota could not open external link: {error}");
                }
            });
            // Same-origin left-aligned 72ch plane as Write body (HTML also caps at 72ch).
            let preview_plane = WritingPlane::new(writing_plane_max_px);
            preview_plane.set_vexpand(true);
            preview_plane.set_margin_start(32);
            preview_plane.set_margin_end(32);
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

        let editor_footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        editor_footer.set_height_request(NATIVE_VISUAL_CONTRACT.footer_height);
        editor_footer.set_css_classes(&["nota-editor-footer"]);
        editor_footer.append(&statistics);
        let modes = gtk::Box::new(gtk::Orientation::Horizontal, 1);
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
        let help = gtk::Button::with_label("?");
        help.set_css_classes(&["nota-help-button"]);
        help.set_tooltip_text(Some("Markdown help"));
        modes.append(&help);
        editor_footer.append(&modes);
        editor_footer.append(&status);
        editor.append(&editor_footer);

        root.append(&sidebar);
        let divider = gtk::Separator::new(gtk::Orientation::Vertical);
        root.append(&divider);
        root.append(&editor);
        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&root));
        overlay.add_overlay(&notification);
        window.set_child(Some(&overlay));

        let create_sender = sender.clone();
        create.connect_clicked(move |_| {
            let _send_result = create_sender.send(AppMsg::QuickCapture);
        });
        let empty_create_sender = sender.clone();
        empty_create.connect_clicked(move |_| {
            let _send_result = empty_create_sender.send(AppMsg::QuickCapture);
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
        let theme_sender = sender.clone();
        theme.connect_clicked(move |_| {
            let _send_result = theme_sender.send(AppMsg::ToggleTheme);
        });
        let diagnostics_sender = sender.clone();
        diagnostics.connect_clicked(move |_| {
            let _send_result = diagnostics_sender.send(AppMsg::RequestDiagnostics);
        });
        let search_for_command = search.clone();
        focus_search.connect_clicked(move |_| {
            search_for_command.grab_focus();
        });
        let help_window = window.clone();
        help.connect_clicked(move |_| {
            show_markdown_help(&help_window);
        });
        let sidebar_navigation_sender = sender.clone();
        sidebar_navigation.connect_clicked(move |_| {
            let _send_result = sidebar_navigation_sender.send(AppMsg::ToggleNavigation);
        });
        let editor_navigation_sender = sender.clone();
        editor_navigation.connect_clicked(move |_| {
            let _send_result = editor_navigation_sender.send(AppMsg::ToggleNavigation);
        });
        let search_sender = sender.clone();
        search.connect_search_changed(move |entry| {
            let _send_result = search_sender.send(AppMsg::EditSearch(entry.text().to_string()));
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
        let title_sender = sender.clone();
        title.connect_changed(move |entry| {
            // GTK4 Entry puts keyboard focus on an inner GtkText, so has_focus() is
            // false while typing. FOCUS_WITHIN lets title edits reach the model.
            if entry_has_input_focus(entry) {
                let _send_result = title_sender.send(AppMsg::UpdateTitle(entry.text().to_string()));
            }
        });
        let tags_sender = sender.clone();
        tags.connect_changed(move |entry| {
            if entry_has_input_focus(entry) {
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
        let content_sender = sender.clone();
        let content_view = content.clone();
        content.buffer().connect_changed(move |buffer| {
            if content_view.has_focus() {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let _send_result = content_sender.send(AppMsg::UpdateContent(text.to_string()));
            }
        });
        connect_formatting_button(&bold, &content, MarkdownCommand::Bold, sender);
        connect_formatting_button(&italic, &content, MarkdownCommand::Italic, sender);
        connect_formatting_button(&strike, &content, MarkdownCommand::Strikethrough, sender);
        connect_formatting_button(&task, &content, MarkdownCommand::TaskList, sender);
        connect_formatting_button(&table, &content, MarkdownCommand::Table, sender);

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
        shortcuts.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<Control>f"),
            Some(gtk::CallbackAction::new(move |_, _| {
                search_for_shortcut.grab_focus();
                gtk::glib::Propagation::Stop
            })),
        ));
        window.add_controller(shortcuts);

        let resize_sender = sender.clone();
        window.connect_notify_local(Some("width"), move |window, _| {
            let width = window.width();
            if width > 0 {
                let _send_result = resize_sender.send(AppMsg::Resize(width as f64));
            }
        });

        install_css();
        Self {
            root,
            sidebar,
            editor,
            sidebar_navigation,
            editor_navigation,
            divider,
            title,
            tags,
            content,
            status,
            statistics,
            notes_count,
            empty_state,
            notification,
            recovery_panel,
            restore_previous,
            create,
            search,
            backup_dot,
            backup_label,
            result_status,
            empty_title,
            empty_copy,
            empty_create,
            tags_pills,
            tag_suggestions,
            edit_tags,
            filter_row,
            filter_chip,
            clear_all,
            theme_label,
            writing,
            surface_row,
            #[cfg(feature = "preview-webkit")]
            preview,
            #[cfg(not(feature = "preview-webkit"))]
            preview_fallback,
            tag_suggestion_count,
            mode_buttons,
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
        let compact = app.viewport == nota_core::responsive_navigation::ViewportClass::Compact;
        self.sidebar_navigation.set_visible(compact);
        self.editor_navigation.set_visible(compact);
        self.divider.set_visible(!compact);
        self.sidebar.set_visible(!compact || app.note_list_visible);
        self.editor.set_visible(!compact || !app.note_list_visible);
        self.sidebar.set_hexpand(compact);
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
        let rendered_notes = list.projection.rows.len();
        self.notes_count.set_text(&rendered_notes.to_string());
        if let Some(status) = &list.result_status {
            self.result_status.set_text(&status.text);
            self.result_status.set_visible(true);
        } else {
            self.result_status.set_visible(false);
        }
        match list.display_state {
            NoteListDisplayState::EmptyCollection => {
                self.empty_state.set_visible(true);
                self.empty_title.set_text("A quiet place for your notes");
                self.empty_copy.set_text("Create a Note to start writing.");
                self.empty_create.set_visible(true);
            }
            NoteListDisplayState::FilteredEmpty => {
                self.empty_state.set_visible(true);
                self.empty_title
                    .set_text(&list.filtered_empty_message.title);
                self.empty_copy.set_text(list.filtered_empty_message.body);
                self.empty_create.set_visible(false);
            }
            NoteListDisplayState::Rows => self.empty_state.set_visible(false),
        }
        if let Some(tag) = app.note_list.active_tag() {
            self.filter_chip.set_label(&format!("#{tag}"));
            self.filter_row.set_visible(true);
        } else {
            self.filter_row.set_visible(false);
        }
        while let Some(child) = self.tags_pills.first_child() {
            self.tags_pills.remove(&child);
        }
        let editing_tags = app.is_editing_tags();
        self.tags.set_visible(editing_tags);
        self.edit_tags.set_visible(!recovering && !editing_tags);
        self.tags_pills.set_visible(!editing_tags);
        if recovering {
            self.title.set_sensitive(false);
            self.tags.set_sensitive(false);
            self.content.set_sensitive(false);
            self.edit_tags.set_sensitive(false);
        } else if let Some(note) = app.workspace.selected_note() {
            if self.title.text().as_str() != note.title {
                self.title.set_text(&note.title);
            }
            let tags = note.tags.join(", ");
            if self.tags.text().as_str() != tags {
                self.tags.set_text(&tags);
            }
            for tag in &note.tags {
                let pill = gtk::Label::new(Some(&format!("#{tag}")));
                pill.set_css_classes(&["nota-note-tags"]);
                self.tags_pills.append(&pill);
            }
            self.edit_tags.set_sensitive(true);
            let buffer = self.content.buffer();
            let current = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            if current.as_str() != note.content {
                buffer.set_text(&note.content);
            }
            self.title.set_sensitive(true);
            self.tags.set_sensitive(true);
            self.content.set_sensitive(true);
            let lines = if note.content.is_empty() {
                0
            } else {
                note.content.lines().count()
            };
            self.statistics.set_text(&format!(
                "{lines} lines · {} words · {} chars",
                note.word_count(),
                note.character_count()
            ));
        } else {
            self.title.set_text("");
            self.tags.set_text("");
            self.content.buffer().set_text("");
            self.title.set_sensitive(false);
            self.tags.set_sensitive(false);
            self.content.set_sensitive(false);
            self.statistics.set_text("0 lines · 0 words · 0 chars");
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
            SaveStatus::Saved => "Saved",
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
        self.surface_row
            .set_homogeneous(surfaces.writing && surfaces.preview);
        self.writing.set_visible(surfaces.writing);
        for (mode, button) in &self.mode_buttons {
            if *mode == app.view_mode {
                button.set_css_classes(&["nota-mode-button", "active"]);
            } else {
                button.set_css_classes(&["nota-mode-button"]);
            }
            if *mode == EditorViewMode::Split {
                button.set_sensitive(!compact);
            }
        }
        #[cfg(feature = "preview-webkit")]
        {
            // Preview widget visibility: show whenever preview surface is on (Preview or Split).
            self.preview.widget().set_visible(surfaces.preview);
            if let Some(plane) = self.preview.widget().parent() {
                plane.set_visible(surfaces.preview);
            }
            if let Some(note) = app.workspace.selected_note() {
                self.preview.load_note(
                    note.display_title(),
                    &note.tags,
                    &note.content,
                    matches!(app.theme, nota_core::transition::ThemePreference::Dark),
                );
            }
        }
        #[cfg(not(feature = "preview-webkit"))]
        {
            self.preview_fallback.set_visible(surfaces.preview);
        }
    }
}

fn command_button(icon: &str, label: &str, shortcut: &str) -> gtk::Button {
    command_button_parts(icon, label, shortcut).0
}

fn command_button_parts(icon: &str, label: &str, shortcut: &str) -> (gtk::Button, gtk::Label) {
    let button = gtk::Button::new();
    button.set_css_classes(&["nota-command"]);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 9);
    let icon = gtk::Label::new(Some(icon));
    icon.set_css_classes(&["nota-command-icon"]);
    let label = gtk::Label::new(Some(label));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    let shortcut = gtk::Label::new(Some(shortcut));
    shortcut.set_css_classes(&["nota-command-shortcut"]);
    row.append(&icon);
    row.append(&label);
    row.append(&shortcut);
    button.set_child(Some(&row));
    (button, label)
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
    command: MarkdownCommand,
    sender: &relm4::Sender<AppMsg>,
) {
    let content = content.clone();
    let sender = sender.clone();
    button.connect_clicked(move |_| {
        let buffer = content.buffer();
        let (start, end) = buffer.selection_bounds().unwrap_or_else(|| {
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
        let _send_result = sender.send(AppMsg::ApplyFormatting { selection, command });
    });
}
