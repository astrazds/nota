use std::cell::RefCell;
use std::rc::Rc;

use nota_app::command::Command;
use nota_app::session::Session;
use relm4::gtk;
use relm4::gtk::prelude::*;

pub(super) fn open_import_file(
    window: &gtk::ApplicationWindow,
    sender: relm4::Sender<Command>,
    session: Rc<RefCell<Session>>,
) {
    let dialog = gtk::FileDialog::builder()
        .title("Import / Restore")
        .modal(true)
        .build();
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("JSON files"));
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    dialog.set_default_filter(Some(&filter));
    let window = window.clone();
    gtk::glib::spawn_future_local(async move {
        let Ok(file) = dialog.open_future(Some(&window)).await else {
            return;
        };
        let failed = |message: String| {
            session.borrow_mut().app.notify_failure(message);
            let _send_result = sender.send(Command::Snapshot {});
        };
        match file.load_contents_future().await {
            Ok((bytes, _etag)) => match String::from_utf8(bytes.to_vec()) {
                Ok(json) => {
                    let _send_result = sender.send(Command::ImportNotebook { json });
                }
                Err(_) => failed("Selected file is not valid UTF-8 JSON".to_string()),
            },
            Err(error) => failed(format!("Could not read the selected file: {error}")),
        }
    });
}

pub(super) fn save_json_file(
    window: &gtk::ApplicationWindow,
    title: &str,
    initial_name: &str,
    json: String,
    on_written: impl FnOnce() + 'static,
    on_error: impl FnOnce(String) + 'static,
) {
    let dialog = gtk::FileDialog::builder()
        .title(title)
        .initial_name(initial_name)
        .modal(true)
        .build();
    let window = window.clone();
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
            Ok(_) => on_written(),
            Err((_json, error)) => on_error(format!("Could not write the selected file: {error}")),
        }
    });
}
