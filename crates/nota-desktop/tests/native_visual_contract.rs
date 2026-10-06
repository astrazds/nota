use nota_desktop::visual_contract::{NATIVE_STYLESHEET, NATIVE_VISUAL_CONTRACT};

#[test]
fn focus_layout_preserves_the_existing_nota_palette() {
    let contract = NATIVE_VISUAL_CONTRACT;
    assert_eq!(contract.sidebar_width, 278);
    assert_eq!(contract.footer_height, 65);
    assert_eq!(contract.editor_measure_px, 600);
    assert_eq!(contract.note_title_font_size_px, 39);
    for (actual, expected) in [
        (contract.light.frame, "#F7F5F1"),
        (contract.light.surface, "#FDFCF9"),
        (contract.light.sidebar, "#F0EDE6"),
        (contract.light.graphite, "#25221F"),
        (contract.dark.frame, "#151311"),
        (contract.dark.surface, "#25221F"),
        (contract.dark.sidebar, "#211F1C"),
        (contract.dark.graphite, "#F7F5F1"),
        (contract.capture, "#FFB340"),
        (contract.signal, "#E7A858"),
    ] {
        assert_eq!(actual, expected);
        assert!(NATIVE_STYLESHEET.contains(expected));
    }
}
