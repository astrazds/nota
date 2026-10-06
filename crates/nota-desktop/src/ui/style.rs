use nota_app::storage::Preferences;
use relm4::gtk;
use relm4::gtk::prelude::*;

pub(super) fn startup_window_size(preferences: &Preferences) -> (i32, i32) {
    (
        preferences.window_width.max(480),
        preferences.window_height.max(480),
    )
}

pub(super) fn install_workspace_fonts(window: &gtk::ApplicationWindow) {
    let Some(font_map) = window.pango_context().font_map() else {
        return;
    };
    for path in nota_desktop::fonts::bundled_font_paths() {
        if let Err(error) = font_map.add_font_file(&path) {
            eprintln!("Nota could not register {}: {error}", path.display());
        }
    }
}

pub(super) fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/net/astrazds/Nota/nota.css");
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a GTK display; run mise run test:gtk"]
    fn gtk_child_controls_resolve_bundled_font_families() {
        gtk::init().expect("the GTK font test requires a display");
        let application = gtk::Application::default();
        application
            .register(None::<&gtk::gio::Cancellable>)
            .expect("GTK test application registers");
        let window = gtk::ApplicationWindow::new(&application);
        install_workspace_fonts(&window);
        let child = gtk::Entry::new();
        window.set_child(Some(&child));
        for family in ["Gelasio", "Source Sans 3", "Source Code Pro"] {
            let mut description = gtk::pango::FontDescription::new();
            description.set_family(family);
            description.set_absolute_size(31.0 * gtk::pango::SCALE as f64);
            let font = child
                .pango_context()
                .load_font(&description)
                .expect("bundled font resolves in a child control");
            let resolved = font.describe();
            println!("Requested {family}; resolved {resolved}");
            assert_eq!(resolved.family().as_deref(), Some(family));
        }
    }
}
