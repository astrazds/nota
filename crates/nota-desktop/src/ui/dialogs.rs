use nota_core::markdown_editing::MARKDOWN_CHEATSHEET_SECTIONS;
use nota_desktop::app::AppMsg;
use relm4::gtk;
use relm4::gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

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
    let mut classes = vec!["nota-root", "nota-dialog"];
    if parent.has_css_class("nota-dark") {
        classes.push("nota-dark");
    }
    dialog.set_css_classes(&classes);
    dialog.set_accessible_role(gtk::AccessibleRole::Dialog);
    dialog.set_hide_on_close(true);

    let escape = gtk::EventControllerKey::new();
    escape.set_propagation_phase(gtk::PropagationPhase::Capture);
    let dialog_weak = dialog.downgrade();
    escape.connect_key_pressed(move |_, keyval, _, _| {
        if keyval != gtk::gdk::Key::Escape {
            return gtk::glib::Propagation::Proceed;
        }
        if let Some(dialog) = dialog_weak.upgrade() {
            dialog.close();
        }
        gtk::glib::Propagation::Stop
    });
    dialog.add_controller(escape);

    let heading = gtk::Label::new(Some(title));
    heading.set_halign(gtk::Align::Start);
    heading.set_hexpand(true);
    heading.set_xalign(0.0);
    heading.set_wrap(true);
    heading.set_css_classes(&["nota-dialog-title"]);
    let titles = gtk::Box::new(gtk::Orientation::Vertical, 4);
    titles.set_hexpand(true);
    titles.set_halign(gtk::Align::Start);
    titles.append(&heading);
    if let Some(subtitle) = subtitle {
        let sub = gtk::Label::new(Some(subtitle));
        sub.set_halign(gtk::Align::Start);
        sub.set_xalign(0.0);
        sub.set_wrap(true);
        sub.set_css_classes(&["nota-dialog-subtitle"]);
        titles.append(&sub);
    }

    let close = gtk::Button::from_icon_name("window-close-symbolic");
    close.set_valign(gtk::Align::Start);
    close.set_tooltip_text(Some("Close"));
    close.set_has_frame(false);
    close.set_css_classes(&["nota-dialog-header-close"]);
    close.update_property(&[gtk::accessible::Property::Label("Close")]);
    let dialog_close = dialog.clone();
    close.connect_clicked(move |_| dialog_close.close());

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    header.append(&titles);
    header.append(&close);
    let handle = gtk::WindowHandle::new();
    handle.set_css_classes(&["nota-dialog-header"]);
    handle.set_child(Some(&header));
    dialog.set_titlebar(Some(&handle));
    dialog
}

fn dialog_close_button(dialog: &gtk::Window) -> gtk::Button {
    let close = gtk::Button::with_label("Close");
    close.set_halign(gtk::Align::End);
    close.set_css_classes(&["nota-dialog-close"]);
    let dialog_close = dialog.clone();
    close.connect_clicked(move |_| dialog_close.close());
    close
}

fn dialog_footer(dialog: &gtk::Window) -> (gtk::Box, gtk::Button) {
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    footer.set_halign(gtk::Align::Fill);
    footer.set_css_classes(&["nota-dialog-footer"]);
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
        &["nota-dialog-field", "last"]
    } else {
        &["nota-dialog-field"]
    });
    let key_label = gtk::Label::new(Some(key));
    key_label.set_halign(gtk::Align::Start);
    key_label.set_xalign(0.0);
    key_label.set_css_classes(&["nota-dialog-key"]);
    let value_label = gtk::Label::new(Some(value));
    value_label.set_halign(gtk::Align::Start);
    value_label.set_hexpand(true);
    value_label.set_xalign(0.0);
    value_label.set_wrap(true);
    value_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    value_label.set_selectable(true);
    value_label.set_tooltip_text(Some(value));
    value_label.set_css_classes(&["nota-dialog-value"]);
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
    heading.set_css_classes(&["nota-cheatsheet-heading"]);
    section.append(&heading);
    for item in items {
        let code = gtk::Label::new(Some(*item));
        code.set_halign(gtk::Align::Fill);
        code.set_xalign(0.0);
        code.set_wrap(true);
        code.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        code.set_selectable(true);
        code.set_css_classes(&["nota-cheatsheet-item"]);
        section.append(&code);
    }
    section
}

pub(super) fn show_about_dialog(
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
    root.set_css_classes(&["nota-dialog-panel"]);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body.set_css_classes(&["nota-dialog-body"]);
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

pub(super) fn show_markdown_help(parent: &gtk::ApplicationWindow) {
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
    root.set_css_classes(&["nota-dialog-panel"]);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body.set_hexpand(true);
    body.set_vexpand(true);
    body.set_css_classes(&["nota-dialog-body"]);

    let columns = gtk::Box::new(gtk::Orientation::Horizontal, 28);
    columns.set_hexpand(true);
    columns.set_halign(gtk::Align::Fill);
    columns.set_homogeneous(true);
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
    scroll.set_css_classes(&["nota-dialog-scroll"]);
    body.append(&scroll);
    let (footer, close) = dialog_footer(&dialog);
    root.append(&body);
    root.append(&footer);
    dialog.set_child(Some(&root));
    dialog.set_default_widget(Some(&close));
    dialog.present();
    close.grab_focus();
}

pub(super) struct ConfirmationRequest {
    pub(super) title: &'static str,
    pub(super) detail: String,
    pub(super) accept_label: &'static str,
    pub(super) accepted: AppMsg,
    pub(super) cancelled: AppMsg,
    pub(super) destructive: bool,
}

pub(super) fn show_confirmation(
    window: &gtk::ApplicationWindow,
    request: ConfirmationRequest,
    sender: &relm4::Sender<AppMsg>,
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
    root.set_css_classes(&["nota-dialog-panel"]);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_css_classes(&["nota-dialog-body"]);
    let copy = gtk::Label::new(Some(&detail));
    copy.set_halign(gtk::Align::Start);
    copy.set_wrap(true);
    copy.set_xalign(0.0);
    copy.set_css_classes(&["nota-dialog-copy"]);
    body.append(&copy);
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    footer.set_halign(gtk::Align::Fill);
    footer.set_css_classes(&["nota-dialog-footer"]);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let cancel = gtk::Button::with_label("Cancel");
    cancel.set_css_classes(&["nota-dialog-cancel"]);
    let accept = gtk::Button::with_label(accept_label);
    if destructive {
        accept.set_css_classes(&["nota-dialog-accept", "danger"]);
    } else {
        accept.set_css_classes(&["nota-dialog-accept", "nota-dialog-close"]);
    }
    footer.append(&spacer);
    footer.append(&cancel);
    footer.append(&accept);
    root.append(&body);
    root.append(&footer);
    dialog.set_child(Some(&root));
    dialog.set_default_widget(Some(&cancel));

    let responded = Rc::new(Cell::new(false));
    let sender = sender.clone();
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
