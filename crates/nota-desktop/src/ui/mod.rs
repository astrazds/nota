use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{
    ComponentBuilder, ComponentController, ComponentParts, ComponentSender, SimpleComponent,
};

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nota_app::command::{Command, Effect};
use nota_app::session::{OpenOptionsJson, Session};
use nota_app::storage::Preferences;
use nota_desktop::APPLICATION_ID;
use uuid::Uuid;

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
    session: Rc<RefCell<Session>>,
    window: gtk::ApplicationWindow,
    note_lists: NoteLists,
    title: gtk::Entry,
    tags: gtk::Entry,
    edit_sequence: Rc<Cell<u64>>,
    selected_id: Rc<Cell<Option<Uuid>>>,
    scheduled_notification_generation: Option<u64>,
}

impl DesktopComponent {
    fn schedule_notification_dismiss(&mut self, sender: &ComponentSender<Self>) {
        let generation = {
            let session = self.session.borrow();
            if session.app.notification.is_none() {
                return;
            }
            session.app.notification_generation()
        };
        if self.scheduled_notification_generation == Some(generation) {
            return;
        }
        self.scheduled_notification_generation = Some(generation);
        let sender = sender.input_sender().clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
            let _send_result = sender.send(Command::DismissNotification { generation });
        });
    }

    fn preferences(&self) -> Preferences {
        let width = self.window.width();
        let height = self.window.height();
        let theme = self.session.borrow().app.theme;
        Preferences {
            theme,
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
    type Input = Command;
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
        let session = Rc::new(RefCell::new(
            Session::open_with_completion(OpenOptionsJson::default(), move || {
                let _ = completion_sender.send(Command::Snapshot {});
            })
            .unwrap_or_else(|error| {
                eprintln!("Nota could not open its profile: {error}");
                std::process::exit(1);
            }),
        ));
        let (width, height) = startup_window_size(session.borrow().preferences());
        window.set_default_size(width, height);
        let edit_sequence = Rc::new(Cell::new(0));
        let selected_id = Rc::new(Cell::new(None));
        let mut note_lists = NoteLists::new(sender.input_sender());
        note_lists.refresh(&session.borrow().app);
        let widgets = DesktopWidgets::new(
            &window,
            &note_lists,
            session.borrow().recovery.as_ref(),
            sender.input_sender(),
            Rc::clone(&session),
            Rc::clone(&edit_sequence),
            Rc::clone(&selected_id),
        );
        let close_session = Rc::clone(&session);
        let close_window = window.clone();
        let close_sender = sender.input_sender().clone();
        window.connect_close_request(move |_| {
            let session = Rc::clone(&close_session);
            let window = close_window.clone();
            let sender = close_sender.clone();
            gtk::glib::idle_add_local_once(move || {
                let mut session = session.borrow_mut();
                match session.flush() {
                    Ok(()) => window.destroy(),
                    Err(error) => {
                        session.app.notify_failure(format!(
                            "Could not save. Keep this window open and retry closing: {error}"
                        ));
                        drop(session);
                        let _send_result = sender.send(Command::Snapshot {});
                    }
                }
            });
            gtk::glib::Propagation::Stop
        });
        let model = DesktopComponent {
            session,
            window,
            note_lists,
            title: widgets.title_input(),
            tags: widgets.tags_input(),
            edit_sequence,
            selected_id,
            scheduled_notification_generation: None,
        };
        let _send_result = sender.input_sender().send(Command::Resize {
            width_px: width as f64,
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        let prompts = matches!(
            message,
            Command::AskDelete { .. }
                | Command::AskPermanentDelete { .. }
                | Command::AskClearDeleted {}
                | Command::ImportNotebook { .. }
        );
        let toggled_theme = matches!(message, Command::ToggleTheme {} | Command::SetTheme { .. });
        let captured = matches!(message, Command::NewNote {});
        let started_edit_tags = matches!(message, Command::StartEditTags {});
        {
            let mut session = self.session.borrow_mut();
            if let Err(error) = session.apply(message) {
                session.app.notify_failure(error.to_string());
            }
            if captured && session.take_focus_title() {
                let title = self.title.clone();
                gtk::glib::idle_add_local_once(move || {
                    title.grab_focus();
                });
            }
        }
        if started_edit_tags {
            let tags = self.tags.clone();
            gtk::glib::idle_add_local_once(move || {
                tags.grab_focus();
            });
        }
        self.schedule_notification_dismiss(&sender);
        if toggled_theme {
            let preferences = self.preferences();
            let saved = self.session.borrow().store.save_preferences(&preferences);
            if let Err(error) = saved {
                self.session
                    .borrow_mut()
                    .app
                    .notify_failure(error.to_string());
            }
        }
        if prompts {
            let prompt = self.session.borrow().frame().confirm;
            if let Some(prompt) = prompt {
                let destructive = matches!(prompt.action.as_str(), "Move" | "Delete" | "Clear All");
                show_confirmation(
                    &self.window,
                    ConfirmationRequest {
                        title: prompt.title,
                        detail: prompt.body,
                        accept_label: prompt.action,
                        accepted: Command::AcceptConfirm {},
                        cancelled: Command::DismissConfirm {},
                        destructive,
                    },
                    sender.input_sender(),
                );
            }
        }
        self.note_lists.refresh(&self.session.borrow().app);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        let mut session = self.session.borrow_mut();
        let frame = session.frame();
        self.edit_sequence.set(frame.edit_sequence);
        self.selected_id
            .set(frame.selected_note.as_ref().map(|note| note.id));
        let preview_html = if frame.layout.show_preview {
            let title = frame
                .selected_note
                .as_ref()
                .map(|note| note.display_title().to_string())
                .unwrap_or_default();
            let content = frame
                .selected_note
                .as_ref()
                .map(|note| note.content.clone())
                .unwrap_or_default();
            match session.apply(Command::Preview { title, content }) {
                Ok(Effect::Preview(html)) => Some(html),
                _ => None,
            }
        } else {
            None
        };
        widgets.refresh(
            &session.app,
            &frame,
            preview_html.as_deref(),
            &self.window,
            session.recovery.as_ref(),
            sender.input_sender(),
        );
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        let preferences = self.preferences();
        let mut session = self.session.borrow_mut();
        if let Err(error) = session.flush() {
            eprintln!("Nota could not flush the latest collection during shutdown: {error}");
        }
        if let Err(error) = session.store.save_preferences(&preferences) {
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
