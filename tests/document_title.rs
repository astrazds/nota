const INDEX_HTML: &str = include_str!("../index.html");

#[test]
fn browser_title_uses_nota_product_identity() {
    assert!(
        INDEX_HTML.contains("<title>Nota - Local-first Markdown notes</title>"),
        "document title should use Nota product language"
    );
    assert!(
        !INDEX_HTML.contains("Apple Notes Clone"),
        "primary user-facing metadata should not mention Apple Notes Clone"
    );
}
