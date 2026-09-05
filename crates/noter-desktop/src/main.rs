use std::cell::Cell;
use std::rc::Rc;

use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};
use uuid::Uuid;

use noter_core::backup::{
    BackupHealth, assess_backup_health, backup_file_name, export_flat_collection_backup,
};
use noter_core::editor_view::EditorViewMode;
use noter_core::markdown_editing::{MARKDOWN_CHEATSHEET_SECTIONS, MarkdownCommand};
use noter_core::note_discovery::HighlightSegment;
use noter_core::note_list_interaction::{NoteListDisplayState, SEARCH_DEBOUNCE_MS};
use noter_core::note_workspace::FocusIntent;
use noter_core::responsive_navigation::WIDE_VIEWPORT_MIN_WIDTH;
use noter_core::transition::{desktop_transition_file_name, export_desktop_transition};
use noter_desktop::APPLICATION_ID;
use noter_desktop::app::{AppModel, AppMsg, NotificationTone, SaveStatus};
use noter_desktop::persistence::PersistenceWorker;
use noter_desktop::selection::gtk_character_range_to_byte_selection;
use noter_desktop::storage::{
    CollectionEnvelope, LoadOutcome, NativeRecovery, NativeStore, Preferences,
};
use noter_desktop::visual_contract::{NATIVE_VISUAL_CONTRACT, writing_plane_max_width_px};
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
    title: gtk::Entry,
    tags: gtk::Entry,
    tag_suggestion_count: Rc<Cell<usize>>,
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
    mode_buttons: Vec<(EditorViewMode, gtk::Button)>,
}

#[derive(Debug, Clone)]
struct NoteRow {
    id: Uuid,
    title_markup: String,
    date: String,
    preview_markup: String,
    tags: Vec<String>,
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
    SelectTag(String),
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
            set_css_classes: &["noter-note-row"],
            #[watch]
            set_class_active: ("selected", self.selected),

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
                            set_use_markup: true,
                            #[watch]
                            set_label: &self.title_markup,
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
                            set_use_markup: true,
                            #[watch]
                            set_label: &self.preview_markup,
                        },
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                        #[watch]
                        set_visible: !self.tags.is_empty(),

                        gtk::Button {
                            set_css_classes: &["noter-note-tags"],
                            #[watch]
                            set_visible: !self.tags.is_empty(),
                            #[watch]
                            set_label: &self
                                .tags
                                .first()
                                .map(|tag| format!("#{tag}"))
                                .unwrap_or_default(),
                            connect_clicked[sender, tag = self.tags.first().cloned()] => move |_| {
                                if let Some(tag) = tag.clone() {
                                    let _send_result = sender.output(NoteRowOutput::SelectTag(tag));
                                }
                            },
                        },
                        gtk::Button {
                            set_css_classes: &["noter-note-tags"],
                            #[watch]
                            set_visible: self.tags.get(1).is_some(),
                            #[watch]
                            set_label: &self
                                .tags
                                .get(1)
                                .map(|tag| format!("#{tag}"))
                                .unwrap_or_default(),
                            connect_clicked[sender, tag = self.tags.get(1).cloned()] => move |_| {
                                if let Some(tag) = tag.clone() {
                                    let _send_result = sender.output(NoteRowOutput::SelectTag(tag));
                                }
                            },
                        },
                        gtk::Button {
                            set_css_classes: &["noter-note-tags"],
                            #[watch]
                            set_visible: self.tags.get(2).is_some(),
                            #[watch]
                            set_label: &self
                                .tags
                                .get(2)
                                .map(|tag| format!("#{tag}"))
                                .unwrap_or_default(),
                            connect_clicked[sender, tag = self.tags.get(2).cloned()] => move |_| {
                                if let Some(tag) = tag.clone() {
                                    let _send_result = sender.output(NoteRowOutput::SelectTag(tag));
                                }
                            },
                        },
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
    fn schedule_notification_dismiss(&self, sender: &ComponentSender<Self>) {
        if self.app.notification.is_none() {
            return;
        }
        let generation = self.app.notification_generation();
        let sender = sender.input_sender().clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
            let _send_result = sender.send(AppMsg::DismissNotification(generation));
        });
    }

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
            let dark = matches!(
                self.app.theme,
                noter_core::transition::ThemePreference::Dark
            );
            let next: Vec<NoteRow> = self
                .app
                .note_list_render_model()
                .projection
                .rows
                .into_iter()
                .map(|row| NoteRow {
                    id: row.id,
                    title_markup: markup_or_plain(&row.title_highlights, &row.display_title),
                    date: row.display_date,
                    preview_markup: markup_or_plain(&row.preview_highlights, &row.preview),
                    tags: row.tags,
                    pinned: row.is_pinned,
                    selected: row.is_selected,
                    dark,
                })
                .collect();
            let mut rows = self.note_rows.guard();
            let update_in_place = note_list_can_update_in_place(
                &(0..rows.len())
                    .filter_map(|index| rows.get(index).map(|row| row.id))
                    .collect::<Vec<_>>(),
                &next.iter().map(|row| row.id).collect::<Vec<_>>(),
            );
            if update_in_place {
                // Keep the existing row widgets so the sidebar does not jump to the
                // top or steal focus when the user selects a Note.
                for (index, row) in next.into_iter().enumerate() {
                    if let Some(current) = rows.get_mut(index) {
                        *current = row;
                    }
                }
            } else {
                rows.clear();
                for row in next {
                    rows.push_back(row);
                }
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
            .title("Nota")
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
                NoteRowOutput::SelectTag(tag) => AppMsg::SelectTag(tag),
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
        let app_title = gtk::Label::new(Some("Nota"));
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
        search.set_css_classes(&["noter-search"]);
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
        hint_title.set_css_classes(&["noter-footer-label"]);
        let hint_copy = gtk::Label::new(Some("\"phrase\"   title:plan   tag:work   is:pinned"));
        hint_copy.set_halign(gtk::Align::Start);
        hint_copy.set_wrap(true);
        hint_box.append(&hint_title);
        hint_box.append(&hint_copy);
        search_hint.set_child(Some(&hint_box));
        let filter_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        filter_row.set_css_classes(&["noter-filter-row"]);
        let filter_prefix = gtk::Label::new(Some("Filtered by"));
        filter_prefix.set_css_classes(&["noter-footer-label"]);
        let filter_chip = gtk::Button::with_label("#tag");
        filter_chip.set_tooltip_text(Some("Clear tag filter"));
        filter_chip.set_css_classes(&["noter-footer-button"]);
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
        let result_status = gtk::Label::new(None);
        result_status.set_halign(gtk::Align::Start);
        result_status.set_wrap(true);
        result_status.set_css_classes(&["noter-result-status"]);
        result_status.set_visible(false);
        sidebar_content.append(&result_status);
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
        let empty_create = gtk::Button::with_label("Create a Note");
        empty_create.set_css_classes(&["noter-footer-button"]);
        empty_state.append(&empty_create);
        sidebar_content.append(&empty_state);
        sidebar_content.append(note_rows.widget());

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
            button.set_css_classes(&["noter-footer-button"]);
        }
        data_actions.append(&export_backup);
        data_actions.append(&import_backup);
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
        let tags_pills = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        tags_pills.set_css_classes(&["noter-tag-pills"]);
        let edit_tags = gtk::Button::with_label("Edit tags");
        edit_tags.set_css_classes(&["noter-footer-button"]);
        header_inner.append(&tags_pills);
        header_inner.append(&edit_tags);
        header_inner.append(&tags);
        let tag_suggestions = gtk::Box::new(gtk::Orientation::Vertical, 0);
        tag_suggestions.set_css_classes(&["noter-tag-suggestions"]);
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

        let surface_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        surface_row.set_hexpand(true);
        surface_row.set_vexpand(true);
        surface_row.set_homogeneous(false);
        surface_row.set_css_classes(&["noter-surface-row"]);
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
            surface_row.append(&preview_fallback);
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
        let empty_create_sender = sender.input_sender().clone();
        empty_create.connect_clicked(move |_| {
            let _send_result = empty_create_sender.send(AppMsg::QuickCapture);
        });
        let edit_tags_sender = sender.input_sender().clone();
        edit_tags.connect_clicked(move |_| {
            let _send_result = edit_tags_sender.send(AppMsg::StartEditTags);
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
        let import_backup_sender = sender.input_sender().clone();
        import_backup.connect_clicked(move |_| {
            let _send_result = import_backup_sender.send(AppMsg::RequestBackupImport);
        });
        let recovery_import_sender = sender.input_sender().clone();
        recovery_import.connect_clicked(move |_| {
            let _send_result = recovery_import_sender.send(AppMsg::RequestBackupImport);
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
            show_markdown_help(&help_window);
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
        let filter_sender = sender.input_sender().clone();
        filter_chip.connect_clicked(move |_| {
            let _send_result = filter_sender.send(AppMsg::ClearTag);
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
        let finish_tags_sender = sender.input_sender().clone();
        let tags_focus = gtk::EventControllerFocus::new();
        tags_focus.connect_leave(move |_| {
            let _send_result = finish_tags_sender.send(AppMsg::FinishEditTags);
        });
        tags.add_controller(tags_focus);
        let tag_suggestion_count = Rc::new(Cell::new(0));
        let tags_key = gtk::EventControllerKey::new();
        let tags_for_key = tags.clone();
        let accept_tag_sender = sender.input_sender().clone();
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
            title: title.clone(),
            tags: tags.clone(),
            tag_suggestion_count,
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
            let quarantine = if self.store.has_quarantined_corrupt_payloads() {
                "Corrupt payload quarantined"
            } else {
                "No corrupt payload quarantine"
            };
            show_about_dialog(
                &self.window,
                env!("CARGO_PKG_VERSION"),
                &self.store.data_dir().display().to_string(),
                &backup,
                quarantine,
            );
            return;
        }
        if matches!(message, AppMsg::RequestBackupExport) {
            match export_flat_collection_backup(self.app.workspace.notes()) {
                Ok(json) => save_json_file(
                    &self.window,
                    "Export Nota Backup",
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
                    "Export for Nota Desktop",
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
        if let AppMsg::ImportBackupJson(_) = &message {
            self.app.apply(message);
            if let Some(preview) = self
                .app
                .pending_backup_import()
                .map(|pending| pending.preview)
            {
                show_confirmation(
                    &self.window,
                    ConfirmationRequest {
                        title: "Merge Import this Backup?",
                        detail: format!(
                            "Import {} notes: {} new, {} replace",
                            preview.total_imported_notes,
                            preview.notes_to_add,
                            preview.notes_to_replace
                        ),
                        accept_label: "Import",
                        accepted: AppMsg::ConfirmBackupImport,
                        cancelled: AppMsg::CancelBackupImport,
                        destructive: false,
                    },
                    &sender,
                );
            }
            self.schedule_notification_dismiss(&sender);
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
                        self.app.set_storage_recovery(false);
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
                        self.app.set_storage_recovery(false);
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
        let confirmed_backup_import = matches!(&message, AppMsg::ConfirmBackupImport);
        let edited_search = matches!(&message, AppMsg::EditSearch(_));
        let captured = matches!(&message, AppMsg::QuickCapture);
        let started_edit_tags = matches!(&message, AppMsg::StartEditTags);
        if self.app.apply(message) {
            self.schedule_save(&sender);
            if confirmed_backup_import {
                self.recovery = None;
            }
        }
        if captured && self.app.workspace.focus_intent() == FocusIntent::NoteTitle {
            let _intent = self.app.workspace.take_focus_intent();
            self.title.grab_focus();
        }
        if started_edit_tags {
            self.tags.grab_focus();
        }
        if edited_search {
            let sender = sender.input_sender().clone();
            gtk::glib::timeout_add_local_once(
                std::time::Duration::from_millis(SEARCH_DEBOUNCE_MS as u64),
                move || {
                    let _send_result = sender.send(AppMsg::CommitSearch);
                },
            );
        }
        self.schedule_notification_dismiss(&sender);
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
                ConfirmationRequest {
                    title: "Move to Recently Deleted?",
                    detail: format!("“{title}” will move to Recently Deleted."),
                    accept_label: "Move",
                    accepted: AppMsg::ConfirmDelete,
                    cancelled: AppMsg::CancelDelete,
                    destructive: true,
                },
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
                let note_label = if count == 1 { "Note" } else { "Notes" };
                show_confirmation(
                    &self.window,
                    ConfirmationRequest {
                        title: "Permanently clear Recently Deleted?",
                        detail: format!(
                            "This will permanently clear {count} recently deleted {note_label}."
                        ),
                        accept_label: "Clear All",
                        accepted: AppMsg::ConfirmClearAll,
                        cancelled: AppMsg::CancelClearAll,
                        destructive: true,
                    },
                    &sender,
                );
            }
        }
        self.refresh_factories();
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        let dark = matches!(
            self.app.theme,
            noter_core::transition::ThemePreference::Dark
        );
        if dark {
            widgets.root.add_css_class("noter-dark");
            self.window.add_css_class("noter-dark");
        } else {
            widgets.root.remove_css_class("noter-dark");
            self.window.remove_css_class("noter-dark");
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
        let backup_health = self.app.backup_health_status(chrono::Utc::now());
        let backup_dot_class = match backup_health {
            BackupHealth::Missing => "missing",
            BackupHealth::Recent { .. } => "recent",
            BackupHealth::Stale { .. } => "stale",
        };
        widgets
            .backup_dot
            .set_css_classes(&["noter-backup-dot", backup_dot_class]);
        widgets
            .backup_label
            .set_label(self.app.backup_health_label(chrono::Utc::now()));
        widgets.recovery_panel.set_visible(self.recovery.is_some());
        let recovering = self.recovery.is_some() || self.app.is_in_storage_recovery();
        widgets.create.set_sensitive(!recovering);
        widgets.search.set_sensitive(!recovering);
        widgets.restore_previous.set_sensitive(
            self.recovery
                .as_ref()
                .and_then(|recovery| recovery.previous_snapshot.as_ref())
                .is_some(),
        );
        widgets
            .clear_all
            .set_visible(!self.app.workspace.recently_deleted_notes().is_empty());
        let list = self.app.note_list_render_model();
        let rendered_notes = list.projection.rows.len();
        widgets.notes_count.set_text(&rendered_notes.to_string());
        if let Some(status) = &list.result_status {
            widgets.result_status.set_text(&status.text);
            widgets.result_status.set_visible(true);
        } else {
            widgets.result_status.set_visible(false);
        }
        match list.display_state {
            NoteListDisplayState::EmptyCollection => {
                widgets.empty_state.set_visible(true);
                widgets.empty_title.set_text("A quiet place for your notes");
                widgets
                    .empty_copy
                    .set_text("Create a Note to start writing.");
                widgets.empty_create.set_visible(true);
            }
            NoteListDisplayState::FilteredEmpty => {
                widgets.empty_state.set_visible(true);
                widgets
                    .empty_title
                    .set_text(&list.filtered_empty_message.title);
                widgets
                    .empty_copy
                    .set_text(list.filtered_empty_message.body);
                widgets.empty_create.set_visible(false);
            }
            NoteListDisplayState::Rows => widgets.empty_state.set_visible(false),
        }
        if let Some(tag) = self.app.note_list.active_tag() {
            widgets.filter_chip.set_label(&format!("#{tag}"));
            widgets.filter_row.set_visible(true);
        } else {
            widgets.filter_row.set_visible(false);
        }
        while let Some(child) = widgets.tags_pills.first_child() {
            widgets.tags_pills.remove(&child);
        }
        let editing_tags = self.app.is_editing_tags();
        widgets.tags.set_visible(editing_tags);
        widgets.edit_tags.set_visible(!recovering && !editing_tags);
        widgets.tags_pills.set_visible(!editing_tags);
        if recovering {
            widgets.title.set_sensitive(false);
            widgets.tags.set_sensitive(false);
            widgets.content.set_sensitive(false);
            widgets.edit_tags.set_sensitive(false);
        } else if let Some(note) = self.app.workspace.selected_note() {
            if widgets.title.text().as_str() != note.title {
                widgets.title.set_text(&note.title);
            }
            let tags = note.tags.join(", ");
            if widgets.tags.text().as_str() != tags {
                widgets.tags.set_text(&tags);
            }
            for tag in &note.tags {
                let pill = gtk::Label::new(Some(&format!("#{tag}")));
                pill.set_css_classes(&["noter-note-tags"]);
                widgets.tags_pills.append(&pill);
            }
            widgets.edit_tags.set_sensitive(true);
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
            widgets.edit_tags.set_visible(false);
        }
        while let Some(child) = widgets.tag_suggestions.first_child() {
            widgets.tag_suggestions.remove(&child);
        }
        if editing_tags && !recovering {
            let suggestions = self.app.tag_suggestions(widgets.tags.text().as_str());
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
                button.set_css_classes(&["noter-tag-suggestion"]);
                let tags_entry = widgets.tags.clone();
                let suggestion_sender = sender.input_sender().clone();
                button.connect_clicked(move |_| {
                    tags_entry.set_text(&completed);
                    let _send_result =
                        suggestion_sender.send(AppMsg::UpdateTags(completed.clone()));
                });
                widgets.tag_suggestions.append(&button);
            }
            widgets
                .tag_suggestions
                .set_visible(self.tag_suggestion_count.get() > 0);
        } else {
            self.tag_suggestion_count.set(0);
            widgets.tag_suggestions.set_visible(false);
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
        widgets
            .surface_row
            .set_homogeneous(surfaces.writing && surfaces.preview);
        widgets.writing.set_visible(surfaces.writing);
        for (mode, button) in &widgets.mode_buttons {
            if *mode == self.app.view_mode {
                button.set_css_classes(&["noter-mode-button", "active"]);
            } else {
                button.set_css_classes(&["noter-mode-button"]);
            }
            if *mode == EditorViewMode::Split {
                button.set_sensitive(!compact);
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
            eprintln!("Nota could not flush the latest collection during shutdown: {error}");
        }
        if let Err(error) = self.store.save_preferences(&self.preferences()) {
            eprintln!("Nota could not save its preferences during shutdown: {error}");
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
            "Restore Nota Desktop Transition"
        } else {
            "Import Nota Backup"
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

fn glib_escape(text: &str) -> String {
    gtk::glib::markup_escape_text(text).to_string()
}

fn highlight_markup(segments: &[HighlightSegment]) -> String {
    segments
        .iter()
        .map(|segment| {
            let text = gtk::glib::markup_escape_text(&segment.text);
            if segment.is_match {
                format!("<span background=\"#FFB340\" foreground=\"#25221F\">{text}</span>")
            } else {
                text.to_string()
            }
        })
        .collect()
}

fn markup_or_plain(segments: &[HighlightSegment], plain: &str) -> String {
    if segments.is_empty() {
        glib_escape(plain)
    } else {
        highlight_markup(segments)
    }
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
    entry.has_focus() || entry.state_flags().contains(gtk::StateFlags::FOCUS_WITHIN)
}

fn note_list_can_update_in_place(current_ids: &[Uuid], next_ids: &[Uuid]) -> bool {
    current_ids == next_ids
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
        (preferences.window_width, preferences.window_height.max(480))
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
    for path in noter_desktop::fonts::bundled_font_paths() {
        if let Err(error) = font_map.add_font_file(&path) {
            eprintln!("Nota could not register {}: {error}", path.display());
        }
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/net/astrazds/Nota/noter.css");
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

fn paper_dialog(
    parent: &gtk::ApplicationWindow,
    title: &str,
    subtitle: Option<&str>,
    width: i32,
    height: i32,
) -> gtk::Window {
    let mut dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(width);
    if height > 0 {
        dialog = dialog.default_height(height);
    }
    let dialog = dialog.build();
    let mut classes = vec!["noter-root", "noter-dialog"];
    if parent.has_css_class("noter-dark") {
        classes.push("noter-dark");
    }
    dialog.set_css_classes(&classes);
    dialog.set_accessible_role(gtk::AccessibleRole::Dialog);
    dialog.set_hide_on_close(true);

    let heading = gtk::Label::new(Some(title));
    heading.set_halign(gtk::Align::Start);
    heading.set_hexpand(true);
    heading.set_xalign(0.0);
    heading.set_wrap(true);
    heading.set_css_classes(&["noter-dialog-title"]);
    let titles = gtk::Box::new(gtk::Orientation::Vertical, 4);
    titles.set_hexpand(true);
    titles.set_halign(gtk::Align::Start);
    titles.append(&heading);
    if let Some(subtitle) = subtitle {
        let sub = gtk::Label::new(Some(subtitle));
        sub.set_halign(gtk::Align::Start);
        sub.set_xalign(0.0);
        sub.set_wrap(true);
        sub.set_css_classes(&["noter-dialog-subtitle"]);
        titles.append(&sub);
    }

    let close = gtk::Button::from_icon_name("window-close-symbolic");
    close.set_valign(gtk::Align::Start);
    close.set_tooltip_text(Some("Close"));
    close.set_has_frame(false);
    close.set_css_classes(&["noter-dialog-header-close"]);
    close.update_property(&[gtk::accessible::Property::Label("Close")]);
    let dialog_close = dialog.clone();
    close.connect_clicked(move |_| dialog_close.close());

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    header.set_css_classes(&["noter-dialog-header"]);
    header.append(&titles);
    header.append(&close);
    let handle = gtk::WindowHandle::new();
    handle.set_child(Some(&header));
    dialog.set_titlebar(Some(&handle));
    dialog
}

fn dialog_close_button(dialog: &gtk::Window) -> gtk::Button {
    let close = gtk::Button::with_label("Close");
    close.set_halign(gtk::Align::End);
    close.set_css_classes(&["noter-dialog-close"]);
    let dialog_close = dialog.clone();
    close.connect_clicked(move |_| dialog_close.close());
    close
}

fn dialog_footer(dialog: &gtk::Window) -> (gtk::Box, gtk::Button) {
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    footer.set_halign(gtk::Align::Fill);
    footer.set_css_classes(&["noter-dialog-footer"]);
    let close = dialog_close_button(dialog);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    footer.append(&spacer);
    footer.append(&close);
    (footer, close)
}

fn meta_field(key: &str, value: &str, last: bool) -> gtk::Box {
    let field = gtk::Box::new(gtk::Orientation::Vertical, 4);
    field.set_css_classes(if last {
        &["noter-dialog-field", "last"]
    } else {
        &["noter-dialog-field"]
    });
    let key_label = gtk::Label::new(Some(key));
    key_label.set_halign(gtk::Align::Start);
    key_label.set_xalign(0.0);
    key_label.set_css_classes(&["noter-dialog-key"]);
    let value_label = gtk::Label::new(Some(value));
    value_label.set_halign(gtk::Align::Start);
    value_label.set_hexpand(true);
    value_label.set_xalign(0.0);
    value_label.set_wrap(true);
    value_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    value_label.set_selectable(true);
    value_label.set_tooltip_text(Some(value));
    value_label.set_css_classes(&["noter-dialog-value"]);
    field.append(&key_label);
    field.append(&value_label);
    field
}

fn cheatsheet_section(title: &str, items: &[&str]) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    section.set_hexpand(true);
    section.set_halign(gtk::Align::Fill);
    let heading = gtk::Label::new(Some(title));
    heading.set_halign(gtk::Align::Start);
    heading.set_xalign(0.0);
    heading.set_css_classes(&["noter-cheatsheet-heading"]);
    section.append(&heading);
    for item in items {
        let code = gtk::Label::new(Some(*item));
        code.set_halign(gtk::Align::Fill);
        code.set_xalign(0.0);
        code.set_wrap(true);
        code.set_selectable(true);
        code.set_css_classes(&["noter-cheatsheet-item"]);
        section.append(&code);
    }
    section
}

fn show_about_dialog(
    parent: &gtk::ApplicationWindow,
    version: &str,
    storage: &str,
    backup: &str,
    recovery: &str,
) {
    let dialog = paper_dialog(
        parent,
        "About Nota",
        Some(&format!("Nota {version}")),
        460,
        0,
    );
    dialog.set_resizable(false);
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_css_classes(&["noter-dialog-panel"]);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body.set_css_classes(&["noter-dialog-body"]);
    body.append(&meta_field("Storage", storage, false));
    body.append(&meta_field("Backup Health", backup, false));
    body.append(&meta_field("Recovery", recovery, true));
    let (footer, close) = dialog_footer(&dialog);
    root.append(&body);
    root.append(&footer);
    dialog.set_child(Some(&root));
    dialog.set_default_widget(Some(&close));
    dialog.present();
    close.grab_focus();
}

fn show_markdown_help(parent: &gtk::ApplicationWindow) {
    let dialog = paper_dialog(
        parent,
        "Markdown syntax",
        Some("Syntax Nota renders in Preview."),
        720,
        520,
    );
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.set_css_classes(&["noter-dialog-panel"]);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body.set_hexpand(true);
    body.set_vexpand(true);
    body.set_css_classes(&["noter-dialog-body"]);

    let columns = gtk::Box::new(gtk::Orientation::Horizontal, 28);
    columns.set_hexpand(true);
    columns.set_halign(gtk::Align::Fill);
    let left = gtk::Box::new(gtk::Orientation::Vertical, 18);
    left.set_hexpand(true);
    left.set_halign(gtk::Align::Fill);
    let right = gtk::Box::new(gtk::Orientation::Vertical, 18);
    right.set_hexpand(true);
    right.set_halign(gtk::Align::Fill);
    let midpoint = MARKDOWN_CHEATSHEET_SECTIONS.len().div_ceil(2);
    for (index, section) in MARKDOWN_CHEATSHEET_SECTIONS.iter().enumerate() {
        let column = if index < midpoint { &left } else { &right };
        column.append(&cheatsheet_section(section.title, section.items));
    }
    columns.append(&left);
    columns.append(&right);
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .child(&columns)
        .build();
    body.append(&scroll);
    let (footer, close) = dialog_footer(&dialog);
    root.append(&body);
    root.append(&footer);
    dialog.set_child(Some(&root));
    dialog.set_default_widget(Some(&close));
    dialog.present();
    close.grab_focus();
}

struct ConfirmationRequest {
    title: &'static str,
    detail: String,
    accept_label: &'static str,
    accepted: AppMsg,
    cancelled: AppMsg,
    destructive: bool,
}

fn show_confirmation(
    window: &gtk::ApplicationWindow,
    request: ConfirmationRequest,
    sender: &ComponentSender<DesktopComponent>,
) {
    let ConfirmationRequest {
        title: message,
        detail,
        accept_label,
        accepted,
        cancelled,
        destructive,
    } = request;
    let dialog = paper_dialog(window, message, None, 440, 0);
    dialog.set_resizable(false);
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_css_classes(&["noter-dialog-panel"]);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_css_classes(&["noter-dialog-body"]);
    let copy = gtk::Label::new(Some(&detail));
    copy.set_halign(gtk::Align::Start);
    copy.set_wrap(true);
    copy.set_xalign(0.0);
    copy.set_css_classes(&["noter-dialog-copy"]);
    body.append(&copy);
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    footer.set_halign(gtk::Align::Fill);
    footer.set_css_classes(&["noter-dialog-footer"]);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let cancel = gtk::Button::with_label("Cancel");
    cancel.set_css_classes(&["noter-dialog-cancel"]);
    let accept = gtk::Button::with_label(accept_label);
    if destructive {
        accept.set_css_classes(&["noter-dialog-accept", "danger"]);
    } else {
        accept.set_css_classes(&["noter-dialog-accept", "noter-dialog-close"]);
    }
    footer.append(&spacer);
    footer.append(&cancel);
    footer.append(&accept);
    root.append(&body);
    root.append(&footer);
    dialog.set_child(Some(&root));
    dialog.set_default_widget(Some(&cancel));

    let responded = Rc::new(Cell::new(false));
    let sender = sender.input_sender().clone();
    let send_once = {
        let responded = responded.clone();
        let sender = sender.clone();
        let dialog = dialog.clone();
        move |message: AppMsg| {
            if responded.replace(true) {
                return;
            }
            dialog.close();
            let _send_result = sender.send(message);
        }
    };
    let cancel_send = send_once.clone();
    let cancelled_on_cancel = cancelled.clone();
    cancel.connect_clicked(move |_| cancel_send(cancelled_on_cancel.clone()));
    let accept_send = send_once.clone();
    accept.connect_clicked(move |_| accept_send(accepted.clone()));
    let close_send = send_once;
    dialog.connect_close_request(move |_| {
        close_send(cancelled.clone());
        gtk::glib::Propagation::Proceed
    });
    dialog.present();
    cancel.grab_focus();
}

fn main() {
    gtk::gio::resources_register_include!("noter.gresource")
        .expect("bundled Nota resources must register");
    let store = match NativeStore::discover() {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Nota could not locate its data directory: {error}");
            return;
        }
    };
    let (collection, recovery) = match store.load_collection() {
        Ok(LoadOutcome::Ready(collection)) => (collection, None),
        Ok(LoadOutcome::Recovery(recovery)) => {
            eprintln!(
                "Nota detected corrupt collection storage: {}",
                recovery.reason
            );
            (CollectionEnvelope::empty(), Some(recovery))
        }
        Err(error) => {
            eprintln!("Nota could not load its collection: {error}");
            (CollectionEnvelope::empty(), None)
        }
    };
    let preferences = store.load_preferences();
    let mut app = AppModel::new(collection, preferences.theme, store.load_backup_health());
    if recovery.is_some() {
        app.set_storage_recovery(true);
    }
    RelmApp::new(APPLICATION_ID).run::<DesktopComponent>((app, store, recovery, preferences));
}

#[cfg(test)]
mod note_list_sync_tests {
    use super::note_list_can_update_in_place;
    use uuid::Uuid;

    #[test]
    fn selecting_a_note_keeps_row_identity_so_widgets_can_stay_mounted() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        assert!(note_list_can_update_in_place(
            &[first, second],
            &[first, second]
        ));
    }

    #[test]
    fn filtering_or_reordering_rebuilds_the_note_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        assert!(!note_list_can_update_in_place(&[first, second], &[second]));
        assert!(!note_list_can_update_in_place(
            &[first, second],
            &[second, first]
        ));
    }
}
