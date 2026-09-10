use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, SimpleComponent};

use nota_core::backup::{
    BackupHealth, assess_backup_health, backup_file_name, export_flat_collection_backup,
};
use nota_core::note_list_interaction::SEARCH_DEBOUNCE_MS;
use nota_core::note_workspace::FocusIntent;
use nota_core::transition::{desktop_transition_file_name, export_desktop_transition};
use nota_desktop::APPLICATION_ID;
use nota_desktop::app::{AppModel, AppMsg};
use nota_desktop::persistence::PersistenceWorker;
use nota_desktop::storage::{
    CollectionEnvelope, LoadOutcome, NativeRecovery, NativeStore, Preferences,
};

mod dialogs;
mod files;
mod note_list;
mod style;
mod workspace;
mod writing_plane;
use workspace::DesktopWidgets;

use dialogs::{ConfirmationRequest, show_about_dialog, show_confirmation};
use files::{ImportKind, open_json_file, save_json_file};
use note_list::NoteLists;
use style::frame_a_startup_window_size;

struct DesktopComponent {
    app: AppModel,
    window: gtk::ApplicationWindow,
    store: NativeStore,
    recovery: Option<NativeRecovery>,
    worker: Option<PersistenceWorker>,
    note_lists: NoteLists,
    title: gtk::Entry,
    tags: gtk::Entry,
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
        let (width, height) = frame_a_startup_window_size(&preferences);
        window.set_default_size(width, height);
        let mut note_lists = NoteLists::new(sender.input_sender());
        note_lists.refresh(&app);
        let widgets = DesktopWidgets::new(
            &window,
            &note_lists,
            recovery.as_ref(),
            sender.input_sender(),
        );
        let model = DesktopComponent {
            app,
            window,
            store: store.clone(),
            recovery,
            worker: Some(PersistenceWorker::start(store)),
            note_lists,
            title: widgets.title_input(),
            tags: widgets.tags_input(),
        };
        let _send_result = sender.input_sender().send(AppMsg::Resize(width as f64));
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
                    sender.input_sender(),
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
                    sender.input_sender(),
                ),
                Err(error) => {
                    self.app.apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            return;
        }
        if matches!(message, AppMsg::RequestBackupImport) {
            open_json_file(&self.window, ImportKind::Backup, sender.input_sender());
            return;
        }
        if matches!(message, AppMsg::RequestTransitionImport) {
            open_json_file(
                &self.window,
                ImportKind::DesktopTransition,
                sender.input_sender(),
            );
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
                    sender.input_sender(),
                );
            }
            self.schedule_notification_dismiss(&sender);
            self.note_lists.refresh(&self.app);
            return;
        }
        if let AppMsg::ImportTransitionJson(json) = &message {
            match self.app.import_transition(json) {
                Ok(()) => {
                    self.schedule_save(&sender);
                    if let Err(error) = self.store.save_preferences(&self.preferences()) {
                        self.app.apply(AppMsg::OperationFailed(error.to_string()));
                    }
                    if let Err(error) = self
                        .store
                        .persist_backup_health(self.app.backup_health.as_ref())
                    {
                        self.app.apply(AppMsg::OperationFailed(error.to_string()));
                    }
                }
                Err(error) => {
                    self.app.apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            self.note_lists.refresh(&self.app);
            return;
        }
        if matches!(message, AppMsg::RestorePreviousSnapshot) {
            if let Some(recovery) = self.recovery.clone() {
                match self.store.restore_previous(&recovery) {
                    Ok(collection) => {
                        self.app.replace_loaded_collection(collection);
                        self.app.set_storage_recovery(false);
                        self.recovery = None;
                        self.note_lists.refresh(&self.app);
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
                        self.note_lists.refresh(&self.app);
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
                sender.input_sender(),
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
                    sender.input_sender(),
                );
            }
        }
        self.note_lists.refresh(&self.app);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        widgets.refresh(
            &self.app,
            &self.window,
            self.recovery.as_ref(),
            sender.input_sender(),
        );
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

pub(super) fn run() {
    gtk::gio::resources_register_include!("nota.gresource")
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
