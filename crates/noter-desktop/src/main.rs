use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, SimpleComponent};
use uuid::Uuid;

use noter_core::backup::{
    BackupHealth, assess_backup_health, backup_file_name, export_flat_collection_backup,
};
use noter_core::editor_view::EditorViewMode;
use noter_core::markdown_editing::MarkdownCommand;
use noter_core::transition::{desktop_transition_file_name, export_desktop_transition};
use noter_desktop::APPLICATION_ID;
use noter_desktop::app::{AppModel, AppMsg, NotificationTone, SaveStatus};
use noter_core::responsive_navigation::WIDE_VIEWPORT_MIN_WIDTH;
use noter_desktop::persistence::PersistenceWorker;
use noter_desktop::selection::gtk_character_range_to_byte_selection;
use noter_desktop::storage::{
    CollectionEnvelope, LoadOutcome, NativeRecovery, NativeStore, Preferences,
};
use noter_desktop::visual_contract::{
    writing_plane_max_width_px, NATIVE_VISUAL_CONTRACT,
};
#[cfg(feature = "preview-webkit")]
use noter_desktop::webkit_preview::SecurePreview;

mod writing_plane;
use writing_plane::WritingPlane;

struct DesktopComponent {
    app: AppModel,
    window: gtk::ApplicationWindow,
    store: NativeStore,
    recovery: Option<NativeRecovery>,
    worker: Option<PersistenceWorker>,
    note_rows: FactoryVecDeque<NoteRow>,
    deleted_rows: FactoryVecDeque<DeletedRow>,
}

struct DesktopWidgets {
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
    clear_all: gtk::Button,
    theme_label: gtk::Label,
    writing: gtk::Box,
    #[cfg(feature = "preview-webkit")]
    preview: SecurePreview,
    /// Non-webkit Preview/Split surface (message stub).
    #[cfg(not(feature = "preview-webkit"))]
    preview_fallback: gtk::Box,
    mode_buttons: Vec<(EditorViewMode, gtk::Button)>,
}

#[derive(Debug, Clone)]
struct NoteRow {
    id: Uuid,
    title: String,
    date: String,
    preview: String,
    tags: String,
    pinned: bool,
    selected: bool,
    /// Propagated onto the GTK popover so absolute theme tokens can match dark mode.
    dark: bool,
}

#[derive(Debug)]
enum NoteRowOutput {
    Select(Uuid),
    TogglePin(Uuid),
    Delete(Uuid),
}

#[relm4::factory]
impl FactoryComponent for NoteRow {
    type Init = Self;
    type Input = ();
    type Output = NoteRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 0,
            set_css_classes: if self.selected {
                &["noter-note-row", "selected"]
            } else {
                &["noter-note-row"]
            },

            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                set_css_classes: &["noter-note-select"],
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(NoteRowOutput::Select(id));
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 3,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,

                        gtk::Label {
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            set_xalign: 0.0,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_css_classes: &["noter-note-title"],
                            #[watch]
                            set_label: &self.title,
                        },
                        gtk::Label {
                            set_css_classes: &["noter-note-pin"],
                            #[watch]
                            set_label: if self.pinned { "◆" } else { "" },
                        },
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 7,

                        gtk::Label {
                            set_css_classes: &["noter-note-date"],
                            #[watch]
                            set_label: &self.date,
                        },
                        gtk::Label {
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            set_xalign: 0.0,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_css_classes: &["noter-note-preview"],
                            #[watch]
                            set_label: &self.preview,
                        },
                    },
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_css_classes: &["noter-note-tags"],
                        #[watch]
                        set_label: &self.tags,
                        #[watch]
                        set_visible: !self.tags.is_empty(),
                    },
                },
            },

            gtk::MenuButton {
                set_icon_name: "view-more-symbolic",
                set_tooltip_text: Some("Note actions"),
                set_css_classes: &["noter-note-actions"],

                #[wrap(Some)]
                set_popover: popover = &gtk::Popover {
                    set_position: gtk::PositionType::Bottom,
                    #[watch]
                    set_css_classes: if self.dark {
                        &["noter-note-actions-popover", "noter-dark"]
                    } else {
                        &["noter-note-actions-popover"]
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        #[watch]
                        set_css_classes: if self.dark {
                            &["noter-note-menu", "noter-dark"]
                        } else {
                            &["noter-note-menu"]
                        },

                        gtk::Button {
                            set_halign: gtk::Align::Fill,
                            #[watch]
                            set_css_classes: if self.dark {
                                &["noter-note-menu-item", "noter-dark"]
                            } else {
                                &["noter-note-menu-item"]
                            },
                            #[watch]
                            set_label: if self.pinned { "Unpin" } else { "Pin" },
                            connect_clicked[sender, id = self.id] => move |_| {
                                let _send_result = sender.output(NoteRowOutput::TogglePin(id));
                            },
                        },
                        gtk::Button {
                            set_halign: gtk::Align::Fill,
                            set_label: "Delete",
                            #[watch]
                            set_css_classes: if self.dark {
                                &["noter-note-menu-item", "destructive-action", "noter-dark"]
                            } else {
                                &["noter-note-menu-item", "destructive-action"]
                            },
                            connect_clicked[sender, id = self.id] => move |_| {
                                let _send_result = sender.output(NoteRowOutput::Delete(id));
                            },
                        },
                    },
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
}

#[derive(Debug, Clone)]
struct DeletedRow {
    id: Uuid,
    title: String,
}

#[derive(Debug)]
enum DeletedRowOutput {
    Restore(Uuid),
    Clear(Uuid),
}

#[relm4::factory]
impl FactoryComponent for DeletedRow {
    type Init = Self;
    type Input = ();
    type Output = DeletedRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_css_classes: &["noter-deleted-row"],

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
                #[watch]
                set_label: &self.title,
            },
            gtk::Button {
                set_label: "Restore",
                set_css_classes: &["noter-small-button"],
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(DeletedRowOutput::Restore(id));
                },
            },
            gtk::Button {
                set_label: "Delete",
                set_css_classes: &["noter-small-button", "danger"],
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(DeletedRowOutput::Clear(id));
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
}

impl DesktopComponent {
    fn schedule_save(&self, sender: &ComponentSender<Self>) {
        if let Some(worker) = &self.worker {
            let _scheduled = worker.schedule(self.app.revision(), self.app.collection());
            let sender = sender.input_sender().clone();
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(325), move || {
                let _send_result = sender.send(AppMsg::FlushPersistence);
            });
        }
    }

    fn refresh_factories(&mut self) {
        {
            let mut rows = self.note_rows.guard();
            rows.clear();
            for row in self.app.note_list_render_model().projection.rows {
                let tags = row
                    .tags
                    .iter()
                    .map(|tag| format!("#{tag}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let preview = if row.uses_match_snippet {
                    format!("Match: {}", row.preview)
                } else {
                    row.preview
                };
                rows.push_back(NoteRow {
                    id: row.id,
                    title: row.display_title,
                    date: row.display_date,
                    preview,
                    tags,
                    pinned: row.is_pinned,
                    selected: row.is_selected,
                    dark: matches!(
                        self.app.theme,
                        noter_core::transition::ThemePreference::Dark
                    ),
                });
            }
        }
        {
            let mut rows = self.deleted_rows.guard();
            rows.clear();
            for note in self.app.workspace.recently_deleted_notes() {
                rows.push_back(DeletedRow {
                    id: note.id,
                    title: note.display_title().to_string(),
                });
            }
        }
    }

    fn preferences(&self) -> Preferences {
        // GTK reports 0×0 before map. `width().max(640)` used to persist 640×480,
        // which opens Compact exclusive-pane (sidebar XOR editor) on next launch.
        let width = self.window.width();
        let height = self.window.height();
        Preferences {
            theme: self.app.theme,
            window_width: if width > 0 {
                width.max(640)
            } else {
                Preferences::default().window_width
            },
            window_height: if height > 0 {
                height.max(480)
            } else {
                Preferences::default().window_height
            },
        }
    }
}

impl SimpleComponent for DesktopComponent {
    type Init = (AppModel, NativeStore, Option<NativeRecovery>, Preferences);
    type Input = AppMsg;
    type Output = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = DesktopWidgets;

    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title("Noter")
            .default_width(1180)
            .default_height(760)
            .build()
    }

    fn init(
        (app, store, recovery, preferences): Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (startup_width, startup_height) = frame_a_startup_window_size(&preferences);
        window.set_default_size(startup_width, startup_height);
        install_workspace_fonts(&window);
        let writing_plane_max_px = writing_plane_max_width_px(measure_ch_width_px(&window));
        let note_rows_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        note_rows_container.set_css_classes(&["noter-note-list"]);
        let note_rows = FactoryVecDeque::builder()
            .launch(note_rows_container)
            .forward(sender.input_sender(), |output| match output {
                NoteRowOutput::Select(id) => AppMsg::SelectNote(id),
                NoteRowOutput::TogglePin(id) => AppMsg::TogglePin(id),
                NoteRowOutput::Delete(id) => AppMsg::RequestDelete(id),
            });
        let deleted_rows_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let deleted_rows = FactoryVecDeque::builder()
            .launch(deleted_rows_container)
            .forward(sender.input_sender(), |output| match output {
                DeletedRowOutput::Restore(id) => AppMsg::RestoreRecentlyDeleted(id),
                DeletedRowOutput::Clear(id) => AppMsg::PermanentlyDelete(id),
            });
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.set_css_classes(&["noter-root"]);

        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar.set_width_request(NATIVE_VISUAL_CONTRACT.sidebar_width);
        sidebar.set_css_classes(&["noter-sidebar"]);

        let sidebar_header = gtk::Box::new(gtk::Orientation::Vertical, 12);
        sidebar_header.set_css_classes(&["noter-sidebar-header"]);
        let identity = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let app_title = gtk::Label::new(Some("Noter"));
        app_title.set_hexpand(true);
        app_title.set_halign(gtk::Align::Start);
        app_title.set_css_classes(&["noter-app-title"]);
        let sidebar_navigation = gtk::Button::with_label("Writing");
        sidebar_navigation.set_tooltip_text(Some("Close the Note List"));
        sidebar_navigation.set_css_classes(&["noter-navigation-button"]);
        identity.append(&app_title);
        identity.append(&sidebar_navigation);
        sidebar_header.append(&identity);

        let commands = gtk::Box::new(gtk::Orientation::Vertical, 2);
        commands.set_css_classes(&["noter-command-list"]);
        let create = command_button("＋", "New Note", "Ctrl N");
        create.add_css_class("noter-command-primary");
        create.set_accessible_role(gtk::AccessibleRole::Button);
        let (theme, theme_label) = command_button_parts("◐", "Dark Theme", "");
        theme.set_tooltip_text(Some("Toggle Light and Dark Theme"));
        let focus_search = command_button("⌕", "Search", "Ctrl F");
        let diagnostics = command_button("ⓘ", "About Noter", "");
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
        search.set_css_classes(&["noter-search"]);
        sidebar_header.append(&search);
        sidebar.append(&sidebar_header);

        let sidebar_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();
        sidebar_scroll.set_css_classes(&["noter-sidebar-scroll"]);
        let sidebar_content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let notes_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        notes_header.set_css_classes(&["noter-section-header"]);
        let notes_label = gtk::Label::new(Some("Notes"));
        notes_label.set_hexpand(true);
        notes_label.set_halign(gtk::Align::Start);
        let notes_count = gtk::Label::new(None);
        notes_count.set_css_classes(&["noter-section-count"]);
        notes_header.append(&notes_label);
        notes_header.append(&notes_count);
        sidebar_content.append(&notes_header);
        let empty_state = gtk::Box::new(gtk::Orientation::Vertical, 4);
        empty_state.set_css_classes(&["noter-empty-state"]);
        let empty_title = gtk::Label::new(Some("A quiet place for your notes"));
        empty_title.set_wrap(true);
        empty_title.set_css_classes(&["noter-empty-title"]);
        let empty_copy = gtk::Label::new(Some("Create a Note to start writing."));
        empty_copy.set_wrap(true);
        empty_copy.set_css_classes(&["noter-empty-copy"]);
        empty_state.append(&empty_title);
        empty_state.append(&empty_copy);
        sidebar_content.append(&empty_state);
        sidebar_content.append(note_rows.widget());

        // Footer chrome: Backup health + Export + Restore only (declutter).
        // Merge Import and Desktop transition export stay available via AppMsg
        // handlers but are intentionally not shown in this row.
        let data_actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        data_actions.set_hexpand(true);
        data_actions.set_halign(gtk::Align::End);
        let export_backup = gtk::Button::with_label("Export");
        export_backup.set_tooltip_text(Some("Export merge Backup"));
        let import_transition = gtk::Button::with_label("Restore");
        import_transition.set_tooltip_text(Some(
            "Restore a desktop transition into an Empty Collection",
        ));
        for button in [&export_backup, &import_transition] {
            button.set_css_classes(&["noter-footer-button"]);
        }
        data_actions.append(&export_backup);
        data_actions.append(&import_transition);

        let deleted_label = gtk::Label::new(Some("Recently Deleted"));
        deleted_label.set_halign(gtk::Align::Start);
        deleted_label.set_hexpand(true);
        let deleted_header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        deleted_header.set_css_classes(&["noter-deleted-header"]);
        deleted_header.append(&deleted_label);
        let clear_all = gtk::Button::with_label("Clear All");
        clear_all.set_halign(gtk::Align::End);
        clear_all.set_css_classes(&["noter-small-button", "danger"]);
        deleted_header.append(&clear_all);
        let deleted_panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        deleted_panel.set_css_classes(&["noter-deleted-panel"]);
        deleted_panel.append(&deleted_header);
        deleted_panel.append(deleted_rows.widget());
        sidebar_content.append(&deleted_panel);
        sidebar_scroll.set_child(Some(&sidebar_content));
        sidebar.append(&sidebar_scroll);

        let sidebar_footer = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        sidebar_footer.set_height_request(NATIVE_VISUAL_CONTRACT.footer_height);
        sidebar_footer.set_css_classes(&["noter-sidebar-footer"]);
        let backup_status = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let backup_dot = gtk::Label::new(Some("●"));
        backup_dot.set_css_classes(&["noter-backup-dot"]);
        let backup_label = gtk::Label::new(Some("Backup"));
        backup_label.set_css_classes(&["noter-footer-label"]);
        backup_status.append(&backup_dot);
        backup_status.append(&backup_label);
        sidebar_footer.append(&backup_status);
        sidebar_footer.append(&data_actions);
        sidebar.append(&sidebar_footer);

        let editor = gtk::Box::new(gtk::Orientation::Vertical, 0);
        editor.set_hexpand(true);
        editor.set_css_classes(&["noter-editor"]);

        let editor_navigation = gtk::Button::with_label("Notes");
        editor_navigation.set_halign(gtk::Align::Start);
        editor_navigation.set_tooltip_text(Some("Open the Note List"));
        editor_navigation.set_css_classes(&["noter-navigation-button"]);

        let editor_header = gtk::Box::new(gtk::Orientation::Vertical, 4);
        editor_header.set_css_classes(&["noter-editor-header"]);
        editor_header.append(&editor_navigation);

        let title = gtk::Entry::builder()
            .placeholder_text("Note Title")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        title.set_css_classes(&["noter-title"]);
        title.set_hexpand(true);
        title.set_halign(gtk::Align::Fill);
        // Single-line title: grow horizontally within the plane (no wrap/ellipsis).
        title.set_truncate_multiline(true);
        let tags = gtk::Entry::builder()
            .placeholder_text("Add tags, separated by commas")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        tags.set_css_classes(&["noter-tags"]);
        tags.set_hexpand(true);
        tags.set_halign(gtk::Align::Fill);
        // Left-aligned 72ch writing plane for title + tags (web note_measure parity).
        let header_inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
        header_inner.set_hexpand(true);
        header_inner.set_halign(gtk::Align::Fill);
        header_inner.append(&title);
        header_inner.append(&tags);
        let header_plane = WritingPlane::new(writing_plane_max_px);
        header_plane.set_child(Some(&header_inner));
        editor_header.append(&header_plane);

        let content = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::WordChar)
            .hexpand(true)
            .vexpand(true)
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        content.set_css_classes(&["noter-writing-surface"]);
        content.set_left_margin(0);
        content.set_right_margin(0);
        content.set_top_margin(20);
        content.set_bottom_margin(32);

        let formatting = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        formatting.set_css_classes(&["noter-formatting-toolbar"]);
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
        status.set_css_classes(&["noter-save-status"]);
        let statistics = gtk::Label::new(Some("0 lines · 0 words · 0 chars"));
        statistics.set_hexpand(true);
        statistics.set_halign(gtk::Align::Start);
        statistics.set_css_classes(&["noter-statistics"]);
        let notification = gtk::Label::new(None);
        notification.set_halign(gtk::Align::End);
        notification.set_valign(gtk::Align::Start);
        notification.set_margin_top(12);
        notification.set_margin_end(16);
        notification.set_wrap(true);
        notification.set_accessible_role(gtk::AccessibleRole::Status);
        notification.set_css_classes(&["noter-notification"]);

        let recovery_panel = gtk::Box::new(gtk::Orientation::Vertical, 10);
        recovery_panel.set_css_classes(&["noter-recovery"]);
        recovery_panel.set_margin_top(20);
        recovery_panel.set_margin_start(32);
        recovery_panel.set_margin_end(32);
        let recovery_title = gtk::Label::new(Some("Noter could not read the saved collection"));
        recovery_title.set_wrap(true);
        recovery_title.update_property(&[gtk::accessible::Property::Label("Storage Recovery")]);
        let recovery_copy = gtk::Label::new(Some(
            "Restore the Previous Snapshot, or preserve the corrupt payload and start with an Empty Collection.",
        ));
        recovery_copy.set_wrap(true);
        let recovery_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let restore_previous = gtk::Button::with_label("Restore Previous Snapshot");
        let start_empty = gtk::Button::with_label("Start Empty");
        recovery_actions.append(&restore_previous);
        recovery_actions.append(&start_empty);
        recovery_panel.append(&recovery_title);
        recovery_panel.append(&recovery_copy);
        recovery_panel.append(&recovery_actions);

        let writing = gtk::Box::new(gtk::Orientation::Vertical, 0);
        writing.set_hexpand(true);
        writing.set_vexpand(true);
        writing.set_css_classes(&["noter-writing"]);
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
        content_scroll.set_css_classes(&["noter-content-scroll"]);
        writing.append(&content_scroll);

        editor.append(&recovery_panel);
        editor.append(&writing);
        #[cfg(feature = "preview-webkit")]
        let preview = {
            let preview = SecurePreview::new(|target| {
                if let Err(error) = gtk::gio::AppInfo::launch_default_for_uri(
                    target.as_str(),
                    None::<&gtk::gio::AppLaunchContext>,
                ) {
                    eprintln!("Noter could not open external link: {error}");
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
            editor.append(&preview_plane);
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
            preview_fallback.set_css_classes(&["noter-preview-fallback"]);
            preview_fallback.set_visible(false);
            let heading = gtk::Label::new(Some("Preview is unavailable in this build"));
            heading.set_halign(gtk::Align::Start);
            heading.set_wrap(true);
            heading.set_css_classes(&["noter-empty-title"]);
            let copy = gtk::Label::new(Some(
                "Write mode still works. Rebuild with the preview-webkit feature (WebKitGTK 6) for rendered Markdown Preview and Split.",
            ));
            copy.set_halign(gtk::Align::Start);
            copy.set_wrap(true);
            preview_fallback.append(&heading);
            preview_fallback.append(&copy);
            editor.append(&preview_fallback);
            preview_fallback
        };

        let editor_footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        editor_footer.set_height_request(NATIVE_VISUAL_CONTRACT.footer_height);
        editor_footer.set_css_classes(&["noter-editor-footer"]);
        editor_footer.append(&statistics);
        let modes = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        modes.set_css_classes(&["noter-mode-group"]);
        let mut mode_buttons = Vec::new();
        for (label, mode) in [
            ("Write", EditorViewMode::Write),
            ("Preview", EditorViewMode::Preview),
            ("Split", EditorViewMode::Split),
        ] {
            let button = gtk::Button::with_label(label);
            if mode == EditorViewMode::Write {
                button.set_css_classes(&["noter-mode-button", "active"]);
            } else {
                button.set_css_classes(&["noter-mode-button"]);
            }
            button.set_sensitive(true);
            let mode_sender = sender.input_sender().clone();
            button.connect_clicked(move |_| {
                let _send_result = mode_sender.send(AppMsg::SetViewMode(mode));
            });
            modes.append(&button);
            mode_buttons.push((mode, button));
        }
        let help = gtk::Button::with_label("?");
        help.set_css_classes(&["noter-help-button"]);
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

        let create_sender = sender.input_sender().clone();
        create.connect_clicked(move |_| {
            let _send_result = create_sender.send(AppMsg::QuickCapture);
        });
        let restore_sender = sender.input_sender().clone();
        restore_previous.connect_clicked(move |_| {
            let _send_result = restore_sender.send(AppMsg::RestorePreviousSnapshot);
        });
        let empty_sender = sender.input_sender().clone();
        start_empty.connect_clicked(move |_| {
            let _send_result = empty_sender.send(AppMsg::StartEmptyAfterRecovery);
        });
        let clear_all_sender = sender.input_sender().clone();
        clear_all.connect_clicked(move |_| {
            let _send_result = clear_all_sender.send(AppMsg::RequestClearAll);
        });
        let export_backup_sender = sender.input_sender().clone();
        export_backup.connect_clicked(move |_| {
            let _send_result = export_backup_sender.send(AppMsg::RequestBackupExport);
        });
        let import_transition_sender = sender.input_sender().clone();
        import_transition.connect_clicked(move |_| {
            let _send_result = import_transition_sender.send(AppMsg::RequestTransitionImport);
        });
        let theme_sender = sender.input_sender().clone();
        theme.connect_clicked(move |_| {
            let _send_result = theme_sender.send(AppMsg::ToggleTheme);
        });
        let diagnostics_sender = sender.input_sender().clone();
        diagnostics.connect_clicked(move |_| {
            let _send_result = diagnostics_sender.send(AppMsg::RequestDiagnostics);
        });
        let search_for_command = search.clone();
        focus_search.connect_clicked(move |_| {
            search_for_command.grab_focus();
        });
        let help_window = window.clone();
        help.connect_clicked(move |_| {
            gtk::AlertDialog::builder()
                .modal(true)
                .message("Markdown shortcuts")
                .detail(
                    "**bold**   *italic*   ~~strike~~\n- [ ] task   # heading   [link](https://…)",
                )
                .buttons(["Close"])
                .cancel_button(0)
                .default_button(0)
                .build()
                .show(Some(&help_window));
        });
        let sidebar_navigation_sender = sender.input_sender().clone();
        sidebar_navigation.connect_clicked(move |_| {
            let _send_result = sidebar_navigation_sender.send(AppMsg::ToggleNavigation);
        });
        let editor_navigation_sender = sender.input_sender().clone();
        editor_navigation.connect_clicked(move |_| {
            let _send_result = editor_navigation_sender.send(AppMsg::ToggleNavigation);
        });
        let search_sender = sender.input_sender().clone();
        search.connect_search_changed(move |entry| {
            let _send_result = search_sender.send(AppMsg::EditSearch(entry.text().to_string()));
            let _send_result = search_sender.send(AppMsg::CommitSearch);
        });
        let title_sender = sender.input_sender().clone();
        title.connect_changed(move |entry| {
            // GTK4 Entry puts keyboard focus on an inner GtkText, so has_focus() is
            // false while typing — use FOCUS_WITHIN so title edits reach the model.
            if entry_has_input_focus(entry) {
                let _send_result = title_sender.send(AppMsg::UpdateTitle(entry.text().to_string()));
            }
        });
        let tags_sender = sender.input_sender().clone();
        tags.connect_changed(move |entry| {
            if entry_has_input_focus(entry) {
                let _send_result = tags_sender.send(AppMsg::UpdateTags(entry.text().to_string()));
            }
        });
        let content_sender = sender.input_sender().clone();
        let content_view = content.clone();
        content.buffer().connect_changed(move |buffer| {
            if content_view.has_focus() {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let _send_result = content_sender.send(AppMsg::UpdateContent(text.to_string()));
            }
        });
        connect_formatting_button(&bold, &content, MarkdownCommand::Bold, &sender);
        connect_formatting_button(&italic, &content, MarkdownCommand::Italic, &sender);
        connect_formatting_button(&strike, &content, MarkdownCommand::Strikethrough, &sender);
        connect_formatting_button(&task, &content, MarkdownCommand::TaskList, &sender);
        connect_formatting_button(&table, &content, MarkdownCommand::Table, &sender);

        let shortcuts = gtk::ShortcutController::new();
        let quick_capture_sender = sender.input_sender().clone();
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

        let resize_sender = sender.input_sender().clone();
        window.connect_notify_local(Some("width"), move |window, _| {
            let width = window.width();
            if width > 0 {
                let _send_result = resize_sender.send(AppMsg::Resize(width as f64));
            }
        });

        install_css();
        let mut model = DesktopComponent {
            app,
            window: window.clone(),
            store: store.clone(),
            recovery,
            worker: Some(PersistenceWorker::start(store)),
            note_rows,
            deleted_rows,
        };
        model.refresh_factories();
        let widgets = DesktopWidgets {
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
            clear_all,
            theme_label,
            writing,
            #[cfg(feature = "preview-webkit")]
            preview,
            #[cfg(not(feature = "preview-webkit"))]
            preview_fallback,
            mode_buttons,
        };
        let _send_result = sender
            .input_sender()
            .send(AppMsg::Resize(startup_width as f64));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        if matches!(message, AppMsg::RequestDiagnostics) {
            let backup = match assess_backup_health(self.app.backup_health, chrono::Utc::now()) {
                BackupHealth::Missing => "No successful Backup export recorded".to_string(),
                BackupHealth::Recent {
                    last_successful_export_at,
                } => format!("Backup current as of {last_successful_export_at}"),
                BackupHealth::Stale {
                    last_successful_export_at,
                } => format!("Backup stale; last export {last_successful_export_at}"),
            };
            gtk::AlertDialog::builder()
                .modal(true)
                .message("About Noter")
                .detail(format!(
                    "Version {}\nStorage: {}\n{backup}",
                    env!("CARGO_PKG_VERSION"),
                    self.store.data_dir().display(),
                ))
                .buttons(["Close"])
                .cancel_button(0)
                .default_button(0)
                .build()
                .show(Some(&self.window));
            return;
        }
        if matches!(message, AppMsg::RequestBackupExport) {
            match export_flat_collection_backup(self.app.workspace.notes()) {
                Ok(json) => save_json_file(
                    &self.window,
                    "Export Noter Backup",
                    &backup_file_name(chrono::Utc::now()),
                    json,
                    AppMsg::BackupExported(chrono::Utc::now()),
                    &sender,
                ),
                Err(error) => {
                    self.app.apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            return;
        }
        if matches!(message, AppMsg::RequestTransitionExport) {
            match export_desktop_transition(
                self.app.workspace.notes(),
                self.app.workspace.recently_deleted_notes(),
                self.app.theme,
                self.app.backup_health,
            ) {
                Ok(json) => save_json_file(
                    &self.window,
                    "Export for Noter Desktop",
                    &desktop_transition_file_name(chrono::Utc::now()),
                    json,
                    AppMsg::OperationSucceeded("Desktop transition exported".to_string()),
                    &sender,
                ),
                Err(error) => {
                    self.app.apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            return;
        }
        if matches!(message, AppMsg::RequestBackupImport) {
            open_json_file(&self.window, false, &sender);
            return;
        }
        if matches!(message, AppMsg::RequestTransitionImport) {
            open_json_file(&self.window, true, &sender);
            return;
        }
        if let AppMsg::ImportBackupJson(json) = &message {
            match self.app.import_backup(json) {
                Ok(()) => self.schedule_save(&sender),
                Err(error) => {
                    self.app.apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            self.refresh_factories();
            return;
        }
        if let AppMsg::ImportTransitionJson(json) = &message {
            match self.app.import_transition(json) {
                Ok(()) => {
                    self.schedule_save(&sender);
                    if let Err(error) = self.store.save_preferences(&self.preferences()) {
                        self.app.apply(AppMsg::OperationFailed(error.to_string()));
                    }
                    if let Some(health) = self.app.backup_health
                        && let Err(error) = self.store.save_backup_health(&health)
                    {
                        self.app.apply(AppMsg::OperationFailed(error.to_string()));
                    }
                }
                Err(error) => {
                    self.app.apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            self.refresh_factories();
            return;
        }
        if matches!(message, AppMsg::RestorePreviousSnapshot) {
            if let Some(recovery) = self.recovery.clone() {
                match self.store.restore_previous(&recovery) {
                    Ok(collection) => {
                        self.app.replace_loaded_collection(collection);
                        self.recovery = None;
                        self.refresh_factories();
                    }
                    Err(error) => {
                        self.app.apply(AppMsg::PersistenceFailed(error.to_string()));
                    }
                }
            }
            return;
        }
        if matches!(message, AppMsg::StartEmptyAfterRecovery) {
            if let Some(recovery) = self.recovery.clone() {
                match self.store.start_empty(&recovery, chrono::Utc::now()) {
                    Ok((collection, _quarantine)) => {
                        self.app.replace_loaded_collection(collection);
                        self.recovery = None;
                        self.refresh_factories();
                    }
                    Err(error) => {
                        self.app.apply(AppMsg::PersistenceFailed(error.to_string()));
                    }
                }
            }
            return;
        }
        if matches!(message, AppMsg::FlushPersistence) {
            if let Some(worker) = &self.worker {
                match worker.flush() {
                    Ok(revision) => {
                        self.app.apply(AppMsg::PersistenceComplete(revision));
                    }
                    Err(error) => {
                        self.app.apply(AppMsg::PersistenceFailed(error.to_string()));
                    }
                }
            }
            return;
        }
        let requested_delete = matches!(&message, AppMsg::RequestDelete(_));
        let requested_clear_all = matches!(&message, AppMsg::RequestClearAll);
        let toggled_theme = matches!(&message, AppMsg::ToggleTheme);
        let backup_exported = matches!(&message, AppMsg::BackupExported(_));
        if self.app.apply(message) {
            self.schedule_save(&sender);
        }
        if toggled_theme && let Err(error) = self.store.save_preferences(&self.preferences()) {
            self.app.apply(AppMsg::OperationFailed(error.to_string()));
        }
        if backup_exported
            && let Some(health) = self.app.backup_health
            && let Err(error) = self.store.save_backup_health(&health)
        {
            self.app.apply(AppMsg::OperationFailed(error.to_string()));
        }
        if requested_delete {
            let title = self
                .app
                .workspace
                .delete_confirmation_title()
                .unwrap_or("New Note");
            show_confirmation(
                &self.window,
                "Move to Recently Deleted?",
                &format!("Move “{title}” to Recently Deleted?"),
                "Move",
                AppMsg::ConfirmDelete,
                AppMsg::CancelDelete,
                &sender,
            );
        }
        if requested_clear_all {
            let count = self
                .app
                .workspace
                .clear_all_recently_deleted_confirmation_count()
                .unwrap_or(0);
            if count > 0 {
                show_confirmation(
                    &self.window,
                    "Permanently clear Recently Deleted?",
                    &format!("Permanently remove {count} recoverable Notes?"),
                    "Clear All",
                    AppMsg::ConfirmClearAll,
                    AppMsg::CancelClearAll,
                    &sender,
                );
            }
        }
        self.refresh_factories();
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let dark = matches!(
            self.app.theme,
            noter_core::transition::ThemePreference::Dark
        );
        if dark {
            widgets.root.add_css_class("noter-dark");
        } else {
            widgets.root.remove_css_class("noter-dark");
        }
        widgets
            .theme_label
            .set_label(if dark { "Light Theme" } else { "Dark Theme" });
        let compact =
            self.app.viewport == noter_core::responsive_navigation::ViewportClass::Compact;
        widgets.sidebar_navigation.set_visible(compact);
        widgets.editor_navigation.set_visible(compact);
        widgets.divider.set_visible(!compact);
        widgets
            .sidebar
            .set_visible(!compact || self.app.note_list_visible);
        widgets
            .editor
            .set_visible(!compact || !self.app.note_list_visible);
        widgets.sidebar.set_hexpand(compact);
        widgets.recovery_panel.set_visible(self.recovery.is_some());
        widgets
            .clear_all
            .set_visible(!self.app.workspace.recently_deleted_notes().is_empty());
        let rendered_notes = self.app.note_list_render_model().projection.rows.len();
        widgets.notes_count.set_text(&rendered_notes.to_string());
        widgets.empty_state.set_visible(rendered_notes == 0);
        if self.recovery.is_some() {
            widgets.title.set_sensitive(false);
            widgets.tags.set_sensitive(false);
            widgets.content.set_sensitive(false);
        } else if let Some(note) = self.app.workspace.selected_note() {
            if widgets.title.text().as_str() != note.title {
                widgets.title.set_text(&note.title);
            }
            let tags = note.tags.join(", ");
            if widgets.tags.text().as_str() != tags {
                widgets.tags.set_text(&tags);
            }
            let buffer = widgets.content.buffer();
            let current = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            if current.as_str() != note.content {
                buffer.set_text(&note.content);
            }
            widgets.title.set_sensitive(true);
            widgets.tags.set_sensitive(true);
            widgets.content.set_sensitive(true);
            let lines = if note.content.is_empty() {
                0
            } else {
                note.content.lines().count()
            };
            widgets.statistics.set_text(&format!(
                "{lines} lines · {} words · {} chars",
                note.word_count(),
                note.character_count()
            ));
        } else {
            widgets.title.set_text("");
            widgets.tags.set_text("");
            widgets.content.buffer().set_text("");
            widgets.title.set_sensitive(false);
            widgets.tags.set_sensitive(false);
            widgets.content.set_sensitive(false);
            widgets.statistics.set_text("0 lines · 0 words · 0 chars");
        }
        widgets.status.set_text(match self.app.save_status {
            SaveStatus::Saved => "Saved",
            SaveStatus::Saving => "Saving…",
            SaveStatus::Failed => "Save failed",
        });
        if let Some(notification) = &self.app.notification {
            widgets.notification.set_text(&notification.message);
            widgets.notification.set_css_classes(&[
                "noter-notification",
                match notification.tone {
                    NotificationTone::Progress => "noter-notification-progress",
                    NotificationTone::Success => "noter-notification-success",
                    NotificationTone::Error => "noter-notification-error",
                },
            ]);
            widgets.notification.set_visible(true);
        } else {
            widgets.notification.set_visible(false);
        }
        let surfaces = self.app.view_mode.surfaces();
        widgets.writing.set_visible(surfaces.writing);
        for (mode, button) in &widgets.mode_buttons {
            if *mode == self.app.view_mode {
                button.set_css_classes(&["noter-mode-button", "active"]);
            } else {
                button.set_css_classes(&["noter-mode-button"]);
            }
        }
        #[cfg(feature = "preview-webkit")]
        {
            // Preview widget visibility: show whenever preview surface is on (Preview or Split).
            widgets.preview.widget().set_visible(surfaces.preview);
            if let Some(plane) = widgets.preview.widget().parent() {
                plane.set_visible(surfaces.preview);
            }
            if let Some(note) = self.app.workspace.selected_note() {
                widgets.preview.load_note(
                    note.display_title(),
                    &note.tags,
                    &note.content,
                    matches!(
                        self.app.theme,
                        noter_core::transition::ThemePreference::Dark
                    ),
                );
            }
        }
        #[cfg(not(feature = "preview-webkit"))]
        {
            widgets.preview_fallback.set_visible(surfaces.preview);
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if let Some(worker) = self.worker.take()
            && let Err(error) = worker.shutdown()
        {
            eprintln!("Noter could not flush the latest collection during shutdown: {error}");
        }
        if let Err(error) = self.store.save_preferences(&self.preferences()) {
            eprintln!("Noter could not save its preferences during shutdown: {error}");
        }
    }
}

fn open_json_file(
    window: &gtk::ApplicationWindow,
    transition: bool,
    sender: &ComponentSender<DesktopComponent>,
) {
    let dialog = gtk::FileDialog::builder()
        .title(if transition {
            "Restore Noter Desktop Transition"
        } else {
            "Import Noter Backup"
        })
        .modal(true)
        .build();
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("JSON files"));
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    dialog.set_default_filter(Some(&filter));
    let window = window.clone();
    let sender = sender.input_sender().clone();
    gtk::glib::spawn_future_local(async move {
        let Ok(file) = dialog.open_future(Some(&window)).await else {
            return;
        };
        match file.load_contents_future().await {
            Ok((bytes, _etag)) => match String::from_utf8(bytes.to_vec()) {
                Ok(json) => {
                    let message = if transition {
                        AppMsg::ImportTransitionJson(json)
                    } else {
                        AppMsg::ImportBackupJson(json)
                    };
                    let _send_result = sender.send(message);
                }
                Err(_) => {
                    let _send_result = sender.send(AppMsg::OperationFailed(
                        "Selected file is not valid UTF-8 JSON".to_string(),
                    ));
                }
            },
            Err(error) => {
                let _send_result = sender.send(AppMsg::OperationFailed(format!(
                    "Could not read the selected file: {error}"
                )));
            }
        }
    });
}

fn save_json_file(
    window: &gtk::ApplicationWindow,
    title: &str,
    initial_name: &str,
    json: String,
    success: AppMsg,
    sender: &ComponentSender<DesktopComponent>,
) {
    let dialog = gtk::FileDialog::builder()
        .title(title)
        .initial_name(initial_name)
        .modal(true)
        .build();
    let window = window.clone();
    let sender = sender.input_sender().clone();
    gtk::glib::spawn_future_local(async move {
        let Ok(file) = dialog.save_future(Some(&window)).await else {
            return;
        };
        match file
            .replace_contents_future(
                json.into_bytes(),
                None,
                false,
                gtk::gio::FileCreateFlags::REPLACE_DESTINATION,
            )
            .await
        {
            Ok(_) => {
                let _send_result = sender.send(success);
            }
            Err((_json, error)) => {
                let _send_result = sender.send(AppMsg::OperationFailed(format!(
                    "Could not write the selected file: {error}"
                )));
            }
        }
    });
}

fn command_button(icon: &str, label: &str, shortcut: &str) -> gtk::Button {
    command_button_parts(icon, label, shortcut).0
}

fn command_button_parts(icon: &str, label: &str, shortcut: &str) -> (gtk::Button, gtk::Label) {
    let button = gtk::Button::new();
    button.set_css_classes(&["noter-command"]);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 9);
    let icon = gtk::Label::new(Some(icon));
    icon.set_css_classes(&["noter-command-icon"]);
    let label = gtk::Label::new(Some(label));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    let shortcut = gtk::Label::new(Some(shortcut));
    shortcut.set_css_classes(&["noter-command-shortcut"]);
    row.append(&icon);
    row.append(&label);
    row.append(&shortcut);
    button.set_child(Some(&row));
    (button, label)
}


/// GTK4 `Entry` keeps keyboard focus on an inner `GtkText`, so `has_focus()` is
/// false while the user is editing. `FOCUS_WITHIN` is set on the Entry itself.
fn entry_has_input_focus(entry: &gtk::Entry) -> bool {
    entry.has_focus()
        || entry
            .state_flags()
            .contains(gtk::StateFlags::FOCUS_WITHIN)
}

fn formatting_button(label: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.set_css_classes(&["noter-toolbar-button"]);
    button.set_tooltip_text(Some(tooltip));
    button
}


/// Prefer Frame A dual-pane on open. Compact exclusive-pane remains available by
/// resizing below the wide breakpoint; do not restore a sub-wide size that was
/// often just the unrealized-window floor (640×480).
fn frame_a_startup_window_size(preferences: &Preferences) -> (i32, i32) {
    let defaults = Preferences::default();
    if f64::from(preferences.window_width) < WIDE_VIEWPORT_MIN_WIDTH {
        (
            defaults.window_width,
            preferences.window_height.max(defaults.window_height),
        )
    } else {
        (
            preferences.window_width,
            preferences.window_height.max(480),
        )
    }
}

/// Pixel width of the CSS `ch` unit (glyph "0") in the notebook body font.
fn measure_ch_width_px(widget: &impl IsA<gtk::Widget>) -> f64 {
    let context = widget.pango_context();
    let mut desc = context
        .font_description()
        .unwrap_or_else(|| gtk::pango::FontDescription::from_string("Sans 14"));
    desc.set_family("Source Sans 3");
    desc.set_size(14 * gtk::pango::SCALE);
    let layout = gtk::pango::Layout::new(&context);
    layout.set_font_description(Some(&desc));
    layout.set_text("0");
    let (width, _) = layout.pixel_size();
    f64::from(width.max(1))
}

fn install_workspace_fonts(window: &gtk::ApplicationWindow) {
    let Some(font_map) = window.pango_context().font_map() else {
        return;
    };
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative_path in [
        "node_modules/@fontsource-variable/source-sans-3/files/source-sans-3-latin-wght-normal.woff2",
        "node_modules/@fontsource-variable/source-sans-3/files/source-sans-3-latin-wght-italic.woff2",
        "node_modules/@fontsource-variable/source-code-pro/files/source-code-pro-latin-wght-normal.woff2",
        "node_modules/@fontsource-variable/source-code-pro/files/source-code-pro-latin-wght-italic.woff2",
    ] {
        let path = workspace.join(relative_path);
        if path.exists()
            && let Err(error) = font_map.add_font_file(&path)
        {
            eprintln!("Noter could not register {}: {error}", path.display());
        }
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/net/astrazds/Noter/noter.css");
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn connect_formatting_button(
    button: &gtk::Button,
    content: &gtk::TextView,
    command: MarkdownCommand,
    sender: &ComponentSender<DesktopComponent>,
) {
    let content = content.clone();
    let sender = sender.input_sender().clone();
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

fn show_confirmation(
    window: &gtk::ApplicationWindow,
    message: &str,
    detail: &str,
    accept_label: &str,
    accepted: AppMsg,
    cancelled: AppMsg,
    sender: &ComponentSender<DesktopComponent>,
) {
    let dialog = gtk::AlertDialog::builder()
        .modal(true)
        .message(message)
        .detail(detail)
        .buttons(["Cancel", accept_label])
        .cancel_button(0)
        .default_button(0)
        .build();
    let sender = sender.input_sender().clone();
    dialog.choose(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |response| {
            let message = if response == Ok(1) {
                accepted
            } else {
                cancelled
            };
            let _send_result = sender.send(message);
        },
    );
}

fn main() {
    gtk::gio::resources_register_include!("noter.gresource")
        .expect("bundled Noter resources must register");
    let store = match NativeStore::discover() {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Noter could not locate its data directory: {error}");
            return;
        }
    };
    let (collection, recovery) = match store.load_collection() {
        Ok(LoadOutcome::Ready(collection)) => (collection, None),
        Ok(LoadOutcome::Recovery(recovery)) => {
            eprintln!(
                "Noter detected corrupt collection storage: {}",
                recovery.reason
            );
            (CollectionEnvelope::empty(), Some(recovery))
        }
        Err(error) => {
            eprintln!("Noter could not load its collection: {error}");
            (CollectionEnvelope::empty(), None)
        }
    };
    let preferences = store.load_preferences();
    let app = AppModel::new(collection, preferences.theme, store.load_backup_health());
    RelmApp::new(APPLICATION_ID).run::<DesktopComponent>((app, store, recovery, preferences));
}
