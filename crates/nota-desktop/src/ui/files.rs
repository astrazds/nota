use nota_desktop::app::AppMsg;
use relm4::gtk;
use relm4::gtk::prelude::*;

#[derive(Clone, Copy)]
pub(super) enum ImportKind {
    Backup,
    DesktopTransition,
}

pub(super) fn open_json_file(
    window: &gtk::ApplicationWindow,
    kind: ImportKind,
    sender: &relm4::Sender<AppMsg>,
) {
    let dialog = gtk::FileDialog::builder()
        .title(match kind {
            ImportKind::DesktopTransition => "Restore Nota Desktop Transition",
            ImportKind::Backup => "Import Nota Backup",
        })
        .modal(true)
        .build();
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("JSON files"));
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    dialog.set_default_filter(Some(&filter));
    let window = window.clone();
    let sender = sender.clone();
    gtk::glib::spawn_future_local(async move {
        let Ok(file) = dialog.open_future(Some(&window)).await else {
            return;
        };
        match file.load_contents_future().await {
            Ok((bytes, _etag)) => match String::from_utf8(bytes.to_vec()) {
                Ok(json) => {
                    let message = match kind {
                        ImportKind::DesktopTransition => AppMsg::ImportTransitionJson(json),
                        ImportKind::Backup => AppMsg::ImportBackupJson(json),
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

pub(super) fn save_json_file(
    window: &gtk::ApplicationWindow,
    title: &str,
    initial_name: &str,
    json: String,
    success: AppMsg,
    sender: &relm4::Sender<AppMsg>,
) {
    let dialog = gtk::FileDialog::builder()
        .title(title)
        .initial_name(initial_name)
        .modal(true)
        .build();
    let window = window.clone();
    let sender = sender.clone();
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
