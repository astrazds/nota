use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, SimpleComponent};
use uuid::Uuid;

use noter_core::backup::{
    BackupHealth, assess_backup_health, backup_file_name, export_flat_collection_backup,
};
#[cfg(feature = "preview-webkit")]
use noter_core::editor_view::EditorViewMode;
use noter_core::markdown_editing::MarkdownCommand;
use noter_core::transition::{desktop_transition_file_name, export_desktop_transition};
use noter_desktop::APPLICATION_ID;
use noter_desktop::app::{AppModel, AppMsg, NotificationTone, SaveStatus};
use noter_desktop::persistence::PersistenceWorker;
use noter_desktop::selection::gtk_character_range_to_byte_selection;
use noter_desktop::storage::{
    CollectionEnvelope, LoadOutcome, NativeRecovery, NativeStore, Preferences,
};
#[cfg(feature = "preview-webkit")]
use noter_desktop::webkit_preview::SecurePreview;

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
    notification: gtk::Label,
    recovery_panel: gtk::Box,
    clear_all: gtk::Button,
    theme: gtk::Button,
    #[cfg(feature = "preview-webkit")]
    writing: gtk::Box,
    #[cfg(feature = "preview-webkit")]
    preview: SecurePreview,
}

#[derive(Debug, Clone)]
struct NoteRow {
    id: Uuid,
    summary: String,
    pinned: bool,
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
            set_spacing: 4,

            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                #[watch]
                set_label: &self.summary,
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(NoteRowOutput::Select(id));
                },
            },
            gtk::Button {
                #[watch]
                set_label: if self.pinned { "Unpin" } else { "Pin" },
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(NoteRowOutput::TogglePin(id));
                },
            },
            gtk::Button {
                set_label: "Delete",
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(NoteRowOutput::Delete(id));
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
            set_spacing: 4,

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                #[watch]
                set_label: &self.title,
            },
            gtk::Button {
                set_label: "Restore",
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(DeletedRowOutput::Restore(id));
                },
            },
            gtk::Button {
                set_label: "Clear",
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
                let detail = [tags, preview]
                    .into_iter()
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(" · ");
                rows.push_back(NoteRow {
                    id: row.id,
                    summary: if detail.is_empty() {
                        row.display_title
                    } else {
                        format!("{}\n{detail}", row.display_title)
                    },
                    pinned: row.is_pinned,
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
        Preferences {
            theme: self.app.theme,
            window_width: self.window.width().max(640),
            window_height: self.window.height().max(480),
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
        window.set_default_size(preferences.window_width, preferences.window_height);
        let note_rows = FactoryVecDeque::builder()
            .launch(gtk::Box::new(gtk::Orientation::Vertical, 4))
            .forward(sender.input_sender(), |output| match output {
                NoteRowOutput::Select(id) => AppMsg::SelectNote(id),
                NoteRowOutput::TogglePin(id) => AppMsg::TogglePin(id),
                NoteRowOutput::Delete(id) => AppMsg::RequestDelete(id),
            });
        let deleted_rows = FactoryVecDeque::builder()
            .launch(gtk::Box::new(gtk::Orientation::Vertical, 4))
            .forward(sender.input_sender(), |output| match output {
                DeletedRowOutput::Restore(id) => AppMsg::RestoreRecentlyDeleted(id),
                DeletedRowOutput::Clear(id) => AppMsg::PermanentlyDelete(id),
            });
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.set_css_classes(&["noter-root"]);

        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 8);
        sidebar.set_width_request(300);
        sidebar.set_margin_top(16);
        sidebar.set_margin_bottom(16);
        sidebar.set_margin_start(16);
        sidebar.set_margin_end(16);

        let sidebar_navigation = gtk::Button::with_label("Back to Writing");
        sidebar_navigation.set_tooltip_text(Some("Close the Note List"));
        sidebar.append(&sidebar_navigation);

        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search Notes")
            .accessible_role(gtk::AccessibleRole::SearchBox)
            .build();
        let create = gtk::Button::with_label("New Note");
        create.set_accessible_role(gtk::AccessibleRole::Button);
        sidebar.append(&search);
        sidebar.append(note_rows.widget());
        sidebar.append(&create);

        let data_actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let export_backup = gtk::Button::with_label("Export");
        export_backup.set_tooltip_text(Some("Export merge Backup"));
        let import_backup = gtk::Button::with_label("Import");
        import_backup.set_tooltip_text(Some("Import merge Backup"));
        let export_transition = gtk::Button::with_label("Desktop Export");
        export_transition.set_tooltip_text(Some("Export exact desktop transition bundle"));
        let import_transition = gtk::Button::with_label("Desktop Restore");
        import_transition.set_tooltip_text(Some(
            "Restore a desktop transition into an Empty Collection",
        ));
        data_actions.append(&export_backup);
        data_actions.append(&import_backup);
        data_actions.append(&export_transition);
        data_actions.append(&import_transition);
        sidebar.append(&data_actions);

        let theme = gtk::Button::with_label("Theme");
        theme.set_tooltip_text(Some("Toggle Light and Dark Theme"));
        sidebar.append(&theme);
        let diagnostics = gtk::Button::with_label("About Noter");
        diagnostics.set_tooltip_text(Some("Show version, storage, and Backup Health"));
        sidebar.append(&diagnostics);
        let deleted_label = gtk::Label::new(Some("Recently Deleted"));
        deleted_label.set_halign(gtk::Align::Start);
        let deleted_header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        deleted_header.append(&deleted_label);
        let clear_all = gtk::Button::with_label("Clear All");
        clear_all.set_halign(gtk::Align::End);
        deleted_header.append(&clear_all);
        sidebar.append(&deleted_header);
        sidebar.append(deleted_rows.widget());

        let editor = gtk::Box::new(gtk::Orientation::Vertical, 10);
        editor.set_hexpand(true);
        editor.set_margin_top(24);
        editor.set_margin_bottom(18);
        editor.set_margin_start(24);
        editor.set_margin_end(24);

        let editor_navigation = gtk::Button::with_label("Notes");
        editor_navigation.set_halign(gtk::Align::Start);
        editor_navigation.set_tooltip_text(Some("Open the Note List"));
        editor.append(&editor_navigation);

        let title = gtk::Entry::builder()
            .placeholder_text("Note Title")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        title.set_css_classes(&["noter-title"]);
        let tags = gtk::Entry::builder()
            .placeholder_text("Tags, separated by commas")
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        let content = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::WordChar)
            .hexpand(true)
            .vexpand(true)
            .accessible_role(gtk::AccessibleRole::TextBox)
            .build();
        content.set_css_classes(&["noter-writing-surface"]);
        let formatting = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let bold = gtk::Button::with_label("Bold");
        let italic = gtk::Button::with_label("Italic");
        let task = gtk::Button::with_label("Task");
        formatting.append(&bold);
        formatting.append(&italic);
        formatting.append(&task);
        let status = gtk::Label::new(Some("Saved"));
        status.set_halign(gtk::Align::End);
        let notification = gtk::Label::new(None);
        notification.set_halign(gtk::Align::Center);
        notification.set_wrap(true);
        notification.set_accessible_role(gtk::AccessibleRole::Status);
        notification.set_css_classes(&["noter-notification"]);

        let recovery_panel = gtk::Box::new(gtk::Orientation::Vertical, 10);
        recovery_panel.set_css_classes(&["noter-recovery"]);
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

        let writing = gtk::Box::new(gtk::Orientation::Vertical, 10);
        writing.set_hexpand(true);
        writing.set_vexpand(true);
        writing.append(&title);
        writing.append(&tags);
        writing.append(&formatting);
        writing.append(&content);

        editor.append(&recovery_panel);
        editor.append(&notification);
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
            preview.widget().set_hexpand(true);
            preview.widget().set_vexpand(true);
            editor.append(preview.widget());
            let modes = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            for (label, mode) in [
                ("Write", EditorViewMode::Write),
                ("Preview", EditorViewMode::Preview),
                ("Split", EditorViewMode::Split),
            ] {
                let button = gtk::Button::with_label(label);
                let mode_sender = sender.input_sender().clone();
                button.connect_clicked(move |_| {
                    let _send_result = mode_sender.send(AppMsg::SetViewMode(mode));
                });
                modes.append(&button);
            }
            editor.append(&modes);
            preview
        };
        editor.append(&status);
        root.append(&sidebar);
        let divider = gtk::Separator::new(gtk::Orientation::Vertical);
        root.append(&divider);
        root.append(&editor);
        window.set_child(Some(&root));

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
        let import_backup_sender = sender.input_sender().clone();
        import_backup.connect_clicked(move |_| {
            let _send_result = import_backup_sender.send(AppMsg::RequestBackupImport);
        });
        let export_transition_sender = sender.input_sender().clone();
        export_transition.connect_clicked(move |_| {
            let _send_result = export_transition_sender.send(AppMsg::RequestTransitionExport);
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
            if entry.has_focus() {
                let _send_result = title_sender.send(AppMsg::UpdateTitle(entry.text().to_string()));
            }
        });
        let tags_sender = sender.input_sender().clone();
        tags.connect_changed(move |entry| {
            if entry.has_focus() {
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
        connect_formatting_button(&task, &content, MarkdownCommand::TaskList, &sender);

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
            let _send_result = resize_sender.send(AppMsg::Resize(window.width() as f64));
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
            notification,
            recovery_panel,
            clear_all,
            theme,
            #[cfg(feature = "preview-webkit")]
            writing,
            #[cfg(feature = "preview-webkit")]
            preview,
        };
        let _send_result = sender
            .input_sender()
            .send(AppMsg::Resize(preferences.window_width as f64));
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
            .theme
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
        } else {
            widgets.title.set_text("");
            widgets.tags.set_text("");
            widgets.content.buffer().set_text("");
            widgets.title.set_sensitive(false);
            widgets.tags.set_sensitive(false);
            widgets.content.set_sensitive(false);
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
        #[cfg(feature = "preview-webkit")]
        {
            let surfaces = self.app.view_mode.surfaces();
            widgets.writing.set_visible(surfaces.writing);
            widgets.preview.widget().set_visible(surfaces.preview);
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
