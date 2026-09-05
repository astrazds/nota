use nota_desktop::visual_contract::{
    NATIVE_STYLESHEET, NATIVE_VISUAL_CONTRACT, writing_plane_max_width_px,
    writing_plane_measure_chars,
};

#[test]
fn native_workspace_frame_matches_the_web_visual_contract() {
    let contract = NATIVE_VISUAL_CONTRACT;

    assert_eq!(contract.sidebar_width, 288);
    assert_eq!(contract.footer_height, 45);
    assert_eq!(contract.editor_measure_chars, 72);
    assert_eq!(contract.note_title_font_size_px, 23);
    assert_eq!(contract.light.frame, "#F7F5F1");
    assert_eq!(contract.light.surface, "#FDFCF9");
    assert_eq!(contract.light.sidebar, "#F0EDE6");
    assert_eq!(contract.light.graphite, "#25221F");
    assert_eq!(contract.dark.frame, "#151311");
    assert_eq!(contract.dark.surface, "#25221F");
    assert_eq!(contract.dark.sidebar, "#211F1C");
    assert_eq!(contract.dark.graphite, "#F7F5F1");
    assert_eq!(contract.capture, "#FFB340");
    assert_eq!(contract.signal, "#E7A858");

    for required_state in [
        ".nota-sidebar",
        ".nota-note-row.selected",
        ".nota-editor-header",
        ".nota-formatting-toolbar",
        ".nota-editor-footer",
        ".nota-root.nota-dark",
        ".nota-writing-plane",
        "button.nota-command-primary",
        "entry.nota-title",
        "button.nota-mode-button.active",
        "window.nota-dialog",
        ".nota-tag-suggestions",
        "button.nota-note-tags",
        ".nota-dialog-field",
        ".nota-cheatsheet-item",
        ".nota-dialog-header",
        "button.nota-dialog-header-close",
    ] {
        assert!(
            NATIVE_STYLESHEET.contains(required_state),
            "missing native style for {required_state}"
        );
    }
}

#[test]
fn paper_writing_plane_and_capture_pulse_tokens_are_declared() {
    assert!(NATIVE_STYLESHEET.contains("--capture: #FFB340"));
    assert!(NATIVE_STYLESHEET.contains("--signal: #E7A858"));
    assert!(NATIVE_STYLESHEET.contains("--frame: #F7F5F1"));
    assert!(NATIVE_STYLESHEET.contains("--surface: #FDFCF9"));
    assert!(NATIVE_STYLESHEET.contains("--sidebar: #F0EDE6"));
    assert!(NATIVE_STYLESHEET.contains("--graphite: #25221F"));
    assert!(NATIVE_STYLESHEET.contains("--frame: #151311"));
    assert!(NATIVE_STYLESHEET.contains("--surface: #25221F"));
    assert!(NATIVE_STYLESHEET.contains("--sidebar: #211F1C"));

    // GTK Stylesheet rejects max-width; measure is enforced in Rust layout.
    assert!(
        !NATIVE_STYLESHEET.contains("max-width: 72ch"),
        "stylesheet must not use GTK-invalid max-width"
    );
    assert!(NATIVE_STYLESHEET.contains("font-size: 23px"));
    assert!(NATIVE_STYLESHEET.contains("min-height: 45px"));

    // Warm Capture Yellow primary for New Note.
    assert!(NATIVE_STYLESHEET.contains("button.nota-command-primary"));
    assert!(NATIVE_STYLESHEET.contains("background: var(--capture)"));

    // Amber active mode segment — no white-pill surface or drop shadow.
    let active_idx = NATIVE_STYLESHEET
        .find("button.nota-mode-button.active")
        .expect("active mode button rule");
    let active_block = &NATIVE_STYLESHEET[active_idx..active_idx + 280];
    assert!(
        active_block.contains("var(--selected)")
            || active_block.contains("background: var(--selected)"),
        "active mode should use amber selected fill"
    );
    assert!(
        !active_block.contains("#FDFCF9"),
        "active mode must not use white-pill surface"
    );
    assert!(
        !active_block.contains("box-shadow: 0 1px 2px"),
        "active mode must not use drop shadow"
    );
    assert!(active_block.contains("box-shadow: none"));
}

#[test]
fn paper_writing_plane_measure_is_wired_from_the_contract() {
    let contract = NATIVE_VISUAL_CONTRACT;
    assert_eq!(writing_plane_measure_chars(), contract.editor_measure_chars);
    assert_eq!(writing_plane_measure_chars(), 72);
    assert_eq!(writing_plane_max_width_px(1.0), 72);
    assert_eq!(writing_plane_max_width_px(8.0), 576);
    assert_eq!(writing_plane_max_width_px(8.4), 605);
    assert!(
        writing_plane_max_width_px(0.0) >= 1,
        "degenerate ch width must still yield a positive px clamp"
    );
    // Dark theme redeclares capture + signal under .nota-root.nota-dark.
    let dark_idx = NATIVE_STYLESHEET
        .find(".nota-root.nota-dark")
        .expect("dark root");
    let dark_block = &NATIVE_STYLESHEET[dark_idx..dark_idx + 520];
    assert!(dark_block.contains("--capture: #FFB340"));
    assert!(dark_block.contains("--signal: #E7A858"));
}

#[test]
fn sidebar_tag_pills_use_the_same_compact_chip_sizing_as_the_writing_surface() {
    let button_idx = NATIVE_STYLESHEET
        .find("button.nota-note-tags")
        .expect("sidebar Tag pills must share the writing-surface chip rule");
    let block = &NATIVE_STYLESHEET[button_idx..button_idx + 280];
    assert!(
        block.contains("min-height: 18px"),
        "sidebar Tag pills must not use GTK default button height"
    );
    assert!(block.contains("font-size: 10px"));
    assert!(block.contains("padding: 2px 8px"));
}
