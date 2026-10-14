use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{
    ComponentBuilder, ComponentController, ComponentParts, ComponentSender, SimpleComponent,
};

use nota_app::app::AppMsg;
use nota_app::session::{OpenOptionsJson, Session};
use nota_app::storage::Preferences;
use nota_core::backup::{BackupHealth, assess_backup_health};
use nota_core::note_list_interaction::SEARCH_DEBOUNCE_MS;
use nota_core::note_workspace::FocusIntent;
use nota_desktop::APPLICATION_ID;

mod dialogs;
mod files;
mod note_list;
mod style;
mod workspace;
mod writing_plane;
use workspace::DesktopWidgets;

use dialogs::{ConfirmationRequest, show_about_dialog, show_confirmation};
use files::{open_import_file, save_json_file};
use note_list::NoteLists;
use style::startup_window_size;

struct DesktopComponent {
    session: Session,
    window: gtk::ApplicationWindow,
    note_lists: NoteLists,
    title: gtk::Entry,
    tags: gtk::Entry,
    scheduled_notification_generation: Option<u64>,
}

impl DesktopComponent {
    fn schedule_notification_dismiss(&mut self, sender: &ComponentSender<Self>) {
        if self.session.app.notification.is_none() {
            return;
        }
        let generation = self.session.app.notification_generation();
        if self.scheduled_notification_generation == Some(generation) {
            return;
        }
        self.scheduled_notification_generation = Some(generation);
        let sender = sender.input_sender().clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
            let _send_result = sender.send(AppMsg::DismissNotification(generation));
        });
    }

    fn preferences(&self) -> Preferences {
        let width = self.window.width();
        let height = self.window.height();
        Preferences {
            theme: self.session.app.theme,
            window_width: if width > 0 {
                width.max(480)
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
    type Init = ();
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
        (): Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Relm4 initializes this component only in the primary GApplication.
        let completion_sender = sender.input_sender().clone();
        let session = Session::open_with_completion(OpenOptionsJson::default(), move || {
            let _ = completion_sender.send(AppMsg::PollPersistence);
        })
        .unwrap_or_else(|error| {
            eprintln!("Nota could not open its profile: {error}");
            std::process::exit(1);
        });
        let (width, height) = startup_window_size(session.preferences());
        window.set_default_size(width, height);
        let mut note_lists = NoteLists::new(sender.input_sender());
        note_lists.refresh(&session.app);
        let widgets = DesktopWidgets::new(
            &window,
            &note_lists,
            session.recovery.as_ref(),
            sender.input_sender(),
        );
        let close_sender = sender.input_sender().clone();
        window.connect_close_request(move |_| {
            let _ = close_sender.send(AppMsg::RequestClose);
            gtk::glib::Propagation::Stop
        });
        let model = DesktopComponent {
            session,
            window,
            note_lists,
            title: widgets.title_input(),
            tags: widgets.tags_input(),
            scheduled_notification_generation: None,
        };
        let _send_result = sender.input_sender().send(AppMsg::Resize(width as f64));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        if matches!(message, AppMsg::RequestClose) {
            match self.session.flush() {
                Ok(()) => self.window.destroy(),
                Err(error) => {
                    self.session.app.apply(AppMsg::OperationFailed(format!(
                        "Could not save. Keep this window open and retry closing: {error}"
                    )));
                }
            }
            return;
        }
        if matches!(message, AppMsg::RequestDiagnostics) {
            let backup =
                match assess_backup_health(self.session.app.backup_health, chrono::Utc::now()) {
                    BackupHealth::Missing => "No successful Backup export recorded".to_string(),
                    BackupHealth::Recent {
                        last_successful_export_at,
                    } => format!("Backup current as of {last_successful_export_at}"),
                    BackupHealth::Stale {
                        last_successful_export_at,
                    } => format!("Backup stale; last export {last_successful_export_at}"),
                };
            let quarantine = if self.session.store.has_quarantined_corrupt_payloads() {
                "Corrupt payload quarantined"
            } else {
                "No corrupt payload quarantine"
            };
            show_about_dialog(
                &self.window,
                env!("CARGO_PKG_VERSION"),
                &self.session.store.data_dir().display().to_string(),
                &backup,
                quarantine,
            );
            return;
        }
        if matches!(message, AppMsg::RequestTagCleanup) {
            let plan = self.session.app.workspace.tag_cleanup_plan();
            if !plan.is_empty() {
                let detail = plan
                    .changes
                    .iter()
                    .map(|change| {
                        format!(
                            "{} -> {}",
                            change.before.join(", "),
                            change.after.join(", ")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                show_confirmation(
                    &self.window,
                    ConfirmationRequest {
                        title: "Clean up Tags?".to_string(),
                        detail,
                        accept_label: "Apply cleanup".to_string(),
                        accepted: AppMsg::ApplyTagCleanup(plan),
                        cancelled: AppMsg::FinishEditTags,
                        destructive: false,
                    },
                    sender.input_sender(),
                );
            }
            return;
        }
        if matches!(message, AppMsg::RequestExport) {
            if self.session.recovery.is_some() {
                self.session.app.apply(AppMsg::OperationFailed(
                    "Resolve Storage Recovery before editing".to_string(),
                ));
                self.schedule_notification_dismiss(&sender);
                self.note_lists.refresh(&self.session.app);
                return;
            }
            match self.session.notebook_export() {
                Ok(exported) => {
                    let name = self.session.snapshot()["suggested_export_name"]
                        .as_str()
                        .unwrap_or("nota-backup.json")
                        .to_string();
                    let health = exported.health;
                    // Gio reports the write before backup-health.json changes. A portal
                    // save has no path the session can write itself.
                    save_json_file(
                        &self.window,
                        "Backup / Export",
                        &name,
                        exported.json,
                        AppMsg::CommitExportedHealth(health),
                        sender.input_sender(),
                    );
                }
                Err(error) => {
                    self.session
                        .app
                        .apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            return;
        }
        if matches!(message, AppMsg::RequestImport) {
            open_import_file(&self.window, sender.input_sender());
            return;
        }
        if let AppMsg::StageImport(json) = message {
            match self
                .session
                .execute(nota_app::session::Request::ImportNotebook { json })
            {
                Ok(_) => {
                    if let Some(copy) = self.session.app.pending_import() {
                        show_confirmation(
                            &self.window,
                            ConfirmationRequest {
                                title: copy.title,
                                detail: copy.body,
                                accept_label: copy.action,
                                accepted: AppMsg::ConfirmBackupImport,
                                cancelled: AppMsg::CancelBackupImport,
                                destructive: false,
                            },
                            sender.input_sender(),
                        );
                    }
                }
                Err(error) => {
                    self.session
                        .app
                        .apply(AppMsg::OperationFailed(error.to_string()));
                }
            }
            self.schedule_notification_dismiss(&sender);
            self.note_lists.refresh(&self.session.app);
            return;
        }
        if let AppMsg::CommitExportedHealth(health) = message {
            if let Err(error) = self.session.record_successful_notes_export(health) {
                self.session
                    .app
                    .apply(AppMsg::OperationFailed(error.to_string()));
            }
            self.schedule_notification_dismiss(&sender);
            self.note_lists.refresh(&self.session.app);
            return;
        }
        if matches!(
            message,
            AppMsg::RestorePreviousSnapshot | AppMsg::StartEmptyAfterRecovery
        ) {
            if let Err(error) = self.session.apply_message(message) {
                self.session
                    .app
                    .apply(AppMsg::OperationFailed(error.to_string()));
            }
            self.schedule_notification_dismiss(&sender);
            self.note_lists.refresh(&self.session.app);
            return;
        }
        let requested_delete = matches!(&message, AppMsg::RequestDelete(_));
        let requested_clear_all = matches!(&message, AppMsg::RequestClearAll);
        let toggled_theme = matches!(&message, AppMsg::ToggleTheme);
        let edited_search = matches!(&message, AppMsg::EditSearch(_));
        let captured = matches!(&message, AppMsg::QuickCapture);
        let started_edit_tags = matches!(&message, AppMsg::StartEditTags);
        if let Err(error) = self.session.apply_message(message) {
            self.session
                .app
                .apply(AppMsg::OperationFailed(error.to_string()));
        }
        if captured && self.session.app.workspace.focus_intent() == FocusIntent::NoteTitle {
            let _intent = self.session.app.workspace.take_focus_intent();
            let title = self.title.clone();
            gtk::glib::idle_add_local_once(move || {
                title.grab_focus();
            });
        }
        if started_edit_tags {
            let tags = self.tags.clone();
            gtk::glib::idle_add_local_once(move || {
                tags.grab_focus();
            });
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
        if toggled_theme
            && let Err(error) = self.session.store.save_preferences(&self.preferences())
        {
            self.session
                .app
                .apply(AppMsg::OperationFailed(error.to_string()));
        }
        if requested_delete {
            let title = self
                .session
                .app
                .workspace
                .delete_confirmation_title()
                .unwrap_or("New Note");
            show_confirmation(
                &self.window,
                ConfirmationRequest {
                    title: "Move to Recently Deleted?".to_string(),
                    detail: format!("“{title}” will move to Recently Deleted."),
                    accept_label: "Move".to_string(),
                    accepted: AppMsg::ConfirmDelete,
                    cancelled: AppMsg::CancelDelete,
                    destructive: true,
                },
                sender.input_sender(),
            );
        }
        if requested_clear_all {
            let count = self
                .session
                .app
                .workspace
                .clear_all_recently_deleted_confirmation_count()
                .unwrap_or(0);
            if count > 0 {
                let note_label = if count == 1 { "Note" } else { "Notes" };
                show_confirmation(
                    &self.window,
                    ConfirmationRequest {
                        title: "Permanently clear Recently Deleted?".to_string(),
                        detail: format!(
                            "This will permanently clear {count} recently deleted {note_label}."
                        ),
                        accept_label: "Clear All".to_string(),
                        accepted: AppMsg::ConfirmClearAll,
                        cancelled: AppMsg::CancelClearAll,
                        destructive: true,
                    },
                    sender.input_sender(),
                );
            }
        }
        self.note_lists.refresh(&self.session.app);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        widgets.refresh(
            &self.session.app,
            &self.window,
            self.session.recovery.as_ref(),
            sender.input_sender(),
        );
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if let Err(error) = self.session.flush() {
            eprintln!("Nota could not flush the latest collection during shutdown: {error}");
        }
        if let Err(error) = self.session.store.save_preferences(&self.preferences()) {
            eprintln!("Nota could not save its preferences during shutdown: {error}");
        }
    }
}

pub(super) fn run() -> gtk::glib::ExitCode {
    gtk::init().expect("GTK must initialize");
    gtk::gio::resources_register_include!("nota.gresource")
        .expect("bundled Nota resources must register");
    let application = relm4::main_application();
    application.set_application_id(Some(APPLICATION_ID));
    let controller = std::rc::Rc::new(std::cell::Cell::new(None));
    let startup_controller = controller.clone();
    application.connect_startup(move |application| {
        let component = ComponentBuilder::<DesktopComponent>::default()
            .launch(())
            .detach();
        application.add_window(component.widget());
        startup_controller.set(Some(component));
    });
    application.connect_activate(|application| {
        if let Some(window) = application.active_window() {
            window.present();
        }
    });
    let status = application.run();
    drop(controller.take());
    // Only dispatch ready component shutdown work. Remote launches have no component.
    let context = gtk::glib::MainContext::ref_thread_default();
    while context.iteration(false) {}
    status
}
