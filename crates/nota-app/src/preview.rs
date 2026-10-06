use std::sync::OnceLock;

use base64::Engine;
use nota_core::markdown_preview::render_markdown_preview_body;
use serde::Deserialize;
use url::Url;

pub const PREVIEW_CSP: &str = "default-src 'none'; script-src 'none'; connect-src 'none'; frame-src 'none'; media-src 'none'; object-src 'none'; img-src data:; font-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'";

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewLayout {
    Reading,
    Split,
    SplitNarrow,
}

fn bundled_font_css() -> &'static str {
    static FONT_CSS: OnceLock<String> = OnceLock::new();
    FONT_CSS.get_or_init(|| {
        let fonts: [(&str, &str, &str, &[u8]); 8] = [
            ("Gelasio", "normal", "400", include_bytes!("../../../assets/fonts/Gelasio-Regular.woff2")),
            ("Gelasio", "italic", "400", include_bytes!("../../../assets/fonts/Gelasio-Italic.woff2")),
            ("Gelasio", "normal", "700", include_bytes!("../../../assets/fonts/Gelasio-Bold.woff2")),
            ("Gelasio", "italic", "700", include_bytes!("../../../assets/fonts/Gelasio-BoldItalic.woff2")),
            ("Source Sans 3", "normal", "200 900", include_bytes!("../../../assets/fonts/source-sans-3-latin-wght-normal.woff2")),
            ("Source Sans 3", "italic", "200 900", include_bytes!("../../../assets/fonts/source-sans-3-latin-wght-italic.woff2")),
            ("Source Code Pro", "normal", "200 900", include_bytes!("../../../assets/fonts/source-code-pro-latin-wght-normal.woff2")),
            ("Source Code Pro", "italic", "200 900", include_bytes!("../../../assets/fonts/source-code-pro-latin-wght-italic.woff2")),
        ];
        fonts.into_iter().map(|(family, style, weight, bytes)| {
            let data = base64::engine::general_purpose::STANDARD.encode(bytes);
            format!("@font-face{{font-family:'{family}';font-style:{style};font-weight:{weight};src:url(data:font/woff2;base64,{data}) format('woff2');}}")
        }).collect()
    })
}

pub fn preview_document(title: &str, markdown: &str, dark: bool, layout: PreviewLayout) -> String {
    let fonts = bundled_font_css();
    let body = render_markdown_preview_body(title, markdown);
    let foreground = if dark { "#F7F5F1" } else { "#332F2A" };
    let background = if dark { "#25221F" } else { "#FDFCF9" };
    let accent = if dark { "#E7B970" } else { "#79501D" };
    let muted = if dark { "#B9B0A4" } else { "#6E685F" };
    let line = if dark { "#51493E" } else { "#D8D1C5" };
    let font_size = match layout {
        PreviewLayout::Reading => 18,
        PreviewLayout::Split => 15,
        PreviewLayout::SplitNarrow => 14,
    };
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"{PREVIEW_CSP}\"><style>{fonts}:root{{color-scheme:{scheme};--capture:#FFB340;--signal:#E7A858}}body{{box-sizing:border-box;max-width:600px;margin:0 auto;padding:0;font:{font_size}px/1.9 'Gelasio',serif;color:{foreground};background:{background};text-align:left;overflow-wrap:anywhere}}p{{margin:0 0 23px}}h1,h2,h3,h4,h5,h6{{font-family:'Source Sans 3',sans-serif}}h2{{font:500 21px/1.4 'Source Sans 3',sans-serif;letter-spacing:-.35px;margin:31px 0 12px}}ul,ol{{padding-left:21px;margin:0 0 23px}}li{{padding-left:3px;margin:6px 0}}li:has(>input[type=checkbox]),li:has(>p>input[type=checkbox]){{list-style:none;position:relative;margin:0 0 0 -21px;padding:3px 0 3px 23px}}input[type=checkbox]{{appearance:none;display:inline-block;vertical-align:baseline;font:inherit;box-sizing:border-box;width:13px;height:13px;margin:0 calc(10px - .25em) 0 -23px;border:0;border-radius:2px;box-shadow:inset 0 0 0 1px {muted};background:transparent}}input[type=checkbox]:checked{{background:{accent};box-shadow:inset 0 0 0 1px {accent}}}@media(forced-colors:active){{input[type=checkbox]{{border:1px solid CanvasText;box-shadow:none}}input[type=checkbox]:checked{{background:Highlight;border-color:Highlight}}}}li:has(>input[type=checkbox]:checked),li:has(>p>input[type=checkbox]:checked){{color:{muted};text-decoration:line-through}}blockquote{{border-left:2px solid {line};color:{muted};padding-left:17px;margin:24px 0;font-style:italic}}h1{{font:600 24px/1.4 'Source Sans 3',sans-serif;margin:0 0 0.75rem}}pre,code{{font:13px/1.6 'Source Code Pro',monospace}}pre{{overflow:auto}}a{{color:{accent}}}img{{max-width:100%}}</style></head><body>{body}</body></html>",
        scheme = if dark { "dark" } else { "light" },
    )
}

pub fn external_navigation_target(uri: &str, user_activated: bool) -> Option<Url> {
    if !user_activated {
        return None;
    }
    let url = Url::parse(uri).ok()?;
    matches!(url.scheme(), "http" | "https" | "mailto").then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_embeds_the_bundled_font_bytes_for_offline_rendering() {
        let html = preview_document(
            "Fonts",
            "Reading **bold** and *italic*.",
            false,
            PreviewLayout::Reading,
        );
        let embedded = html
            .split("src:url(data:font/woff2;base64,")
            .skip(1)
            .map(|face| {
                base64::engine::general_purpose::STANDARD
                    .decode(face.split(')').next().unwrap())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(embedded.len(), 8);
        assert!(embedded.iter().all(|font| font.starts_with(b"wOF2")));
        assert_eq!(
            embedded[0],
            include_bytes!("../../../assets/fonts/Gelasio-Regular.woff2")
        );
        assert!(html.contains("font-src data:"));
        assert!(html.contains("<p>Reading <strong>bold</strong> and <em>italic</em>.</p>"));
    }

    #[test]
    fn preview_document_blocks_active_and_remote_content() {
        let html = preview_document(
            "Safe",
            "<script>alert(1)</script>\n\n![remote](https://example.com/a.png)",
            false,
            PreviewLayout::Reading,
        );

        assert!(html.contains("default-src 'none'"));
        assert!(html.contains("script-src 'none'"));
        assert!(html.contains("img-src data:"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("aria-label=\"Note Metadata\""));
        assert!(!html.contains("<h1>Safe</h1>"));
    }

    #[test]
    fn native_heading_replaces_only_the_matching_leading_markdown_heading() {
        let html = preview_document(
            "Notebook",
            "# Notebook\n\nBody\n\n# Another heading",
            false,
            PreviewLayout::Reading,
        );
        assert!(!html.contains("<h1>Notebook</h1>"));
        assert!(html.contains("<p>Body</p>"));
        assert!(html.contains("<h1>Another heading</h1>"));
        let different = preview_document(
            "Notebook",
            "# Introduction\n\nBody",
            true,
            PreviewLayout::Split,
        );
        assert!(different.contains("<h1>Introduction</h1>"));
    }

    #[test]
    fn external_navigation_requires_user_activation_and_an_allowed_scheme() {
        assert!(external_navigation_target("https://example.com", true).is_some());
        assert!(external_navigation_target("mailto:hello@example.com", true).is_some());
        assert!(external_navigation_target("https://example.com", false).is_none());
        assert!(external_navigation_target("file:///etc/passwd", true).is_none());
        assert!(external_navigation_target("javascript:alert(1)", true).is_none());
    }
}
