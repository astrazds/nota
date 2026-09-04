use noter_core::markdown_preview::render_markdown_preview_body;
use url::Url;

pub const PREVIEW_CSP: &str = "default-src 'none'; script-src 'none'; connect-src 'none'; frame-src 'none'; media-src 'none'; object-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'";

pub fn preview_document(title: &str, tags: &[String], markdown: &str, dark: bool) -> String {
    let body = render_markdown_preview_body(title, markdown);
    let title = escape_html(title);
    let tags = tags
        .iter()
        .map(|tag| format!("<span class=\"tag\">{}</span>", escape_html(tag)))
        .collect::<String>();
    // Frame A surface + left-aligned 72ch measure (web preview_body_text / note_measure).
    let foreground = if dark { "#F7F5F1" } else { "#332F2A" };
    let background = if dark { "#25221F" } else { "#FDFCF9" };
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"{PREVIEW_CSP}\"><style>:root{{color-scheme:{scheme};--capture:#FFB340;--signal:#E7A858}}body{{box-sizing:border-box;max-width:72ch;margin:0;padding:1.25rem 2rem;font:14px/1.65 'Source Sans 3',sans-serif;color:{foreground};background:{background};text-align:left}}h1{{font:600 23px/1.3 'Source Sans 3',sans-serif;margin:0 0 0.75rem}}pre,code{{font-family:'Source Code Pro',monospace}}a{{color:#a86400}}img{{max-width:100%}}.tag{{display:inline-block;margin:0 .4rem 1rem 0;padding:.15rem .5rem;border-radius:999px;background:rgba(231,168,88,0.20);color:#79501D}}</style></head><body><h1>{title}</h1><div aria-label=\"Note Metadata\">{tags}</div>{body}</body></html>",
        scheme = if dark { "dark" } else { "light" },
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
    fn preview_document_blocks_active_and_remote_content() {
        let html = preview_document(
            "Safe",
            &["private".to_string()],
            "<script>alert(1)</script>\n\n![remote](https://example.com/a.png)",
            false,
        );

        assert!(html.contains("default-src 'none'"));
        assert!(html.contains("script-src 'none'"));
        assert!(html.contains("img-src data:"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("aria-label=\"Note Metadata\""));
        assert!(html.contains("max-width:72ch"));
        assert!(html.contains("margin:0"));
        assert!(!html.contains("margin:0 auto"));
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
