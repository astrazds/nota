const INDEX_HTML: &str = include_str!("../index.html");

#[test]
fn browser_title_uses_noter_product_identity() {
    assert!(
        INDEX_HTML.contains("<title>Noter - Local-first Markdown notes</title>"),
        "document title should use Noter product language"
    );
    assert!(
        !INDEX_HTML.contains("Apple Notes Clone"),
        "primary user-facing metadata should not mention Apple Notes Clone"
    );
}
