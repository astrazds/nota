use noter_desktop::visual_contract::{NATIVE_STYLESHEET, NATIVE_VISUAL_CONTRACT};

#[test]
fn native_workspace_frame_matches_the_web_visual_contract() {
    let contract = NATIVE_VISUAL_CONTRACT;

    assert_eq!(contract.sidebar_width, 288);
    assert_eq!(contract.footer_height, 45);
    assert_eq!(contract.editor_measure_chars, 72);
    assert_eq!(contract.light.frame, "#F7F5F1");
    assert_eq!(contract.light.surface, "#FDFCF9");
    assert_eq!(contract.light.sidebar, "#F0EDE6");
    assert_eq!(contract.light.graphite, "#25221F");
    assert_eq!(contract.dark.frame, "#151311");
    assert_eq!(contract.dark.surface, "#25221F");
    assert_eq!(contract.dark.sidebar, "#211F1C");
    assert_eq!(contract.signal, "#E7A858");

    for required_state in [
        ".noter-sidebar",
        ".noter-note-row.selected",
        ".noter-editor-header",
        ".noter-formatting-toolbar",
        ".noter-editor-footer",
        ".noter-root.noter-dark",
    ] {
        assert!(
            NATIVE_STYLESHEET.contains(required_state),
            "missing native style for {required_state}"
        );
    }
}
