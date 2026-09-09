use nota_core::responsive_navigation::WIDE_VIEWPORT_MIN_WIDTH;
use nota_desktop::storage::Preferences;
use relm4::gtk;
use relm4::gtk::prelude::*;

/// Prefer Frame A dual-pane on open. Compact exclusive-pane remains available by
/// resizing below the wide breakpoint; do not restore a sub-wide size that was
/// often just the unrealized-window floor (640×480).
pub(super) fn frame_a_startup_window_size(preferences: &Preferences) -> (i32, i32) {
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
pub(super) fn measure_ch_width_px(widget: &impl IsA<gtk::Widget>) -> f64 {
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
