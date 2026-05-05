use pulldown_cmark::{html, Event, Options, Parser};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

pub fn render_markdown_preview(title: &str, content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let safe_title = escape_html(title);
    let mut html_output = format!("<h1 class=\"text-3xl font-bold mb-4\">{safe_title}</h1>");
    let parser = Parser::new_ext(content, options).map(|event| match event {
        Event::Html(raw_html) | Event::InlineHtml(raw_html) => Event::Text(raw_html),
        _ => event,
    });
    html::push_html(&mut html_output, parser);
    sanitize_preview_html(&html_output)
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(any(test, target_arch = "wasm32"))]
pub fn is_safe_preview_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return true;
    }

    let normalised = trimmed.to_ascii_lowercase();
    normalised.starts_with("http://")
        || normalised.starts_with("https://")
        || normalised.starts_with("mailto:")
        || normalised.starts_with('#')
        || normalised.starts_with('/')
        || normalised.starts_with("./")
        || normalised.starts_with("../")
}

#[cfg(not(target_arch = "wasm32"))]
fn sanitize_preview_html(raw_html: &str) -> String {
    raw_html.to_string()
}

#[cfg(target_arch = "wasm32")]
fn sanitize_preview_html(raw_html: &str) -> String {
    let Some(win) = window() else {
        return raw_html.to_string();
    };
    let Some(doc) = win.document() else {
        return raw_html.to_string();
    };
    let Ok(container) = doc.create_element("div") else {
        return raw_html.to_string();
    };

    container.set_inner_html(raw_html);

    if let Ok(nodes) = container.query_selector_all("*") {
        for index in 0..nodes.length() {
            let Some(node) = nodes.item(index) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<web_sys::Element>() else {
                continue;
            };

            let tag = element.tag_name().to_ascii_lowercase();
            if matches!(
                tag.as_str(),
                "script" | "style" | "iframe" | "object" | "embed" | "link" | "meta"
            ) {
                element.remove();
                continue;
            }

            if tag == "input" && element.get_attribute("type").as_deref() != Some("checkbox") {
                element.remove();
                continue;
            }

            let attrs = element.get_attribute_names();
            for attr_index in 0..attrs.length() {
                let Some(attr_name) = attrs.get(attr_index).as_string() else {
                    continue;
                };
                let attr_name_lower = attr_name.to_ascii_lowercase();

                if attr_name_lower.starts_with("on")
                    || attr_name_lower == "style"
                    || attr_name_lower == "srcdoc"
                {
                    let _ = element.remove_attribute(&attr_name);
                    continue;
                }

                if (attr_name_lower == "href" || attr_name_lower == "src")
                    && element
                        .get_attribute(&attr_name)
                        .is_some_and(|value| !is_safe_preview_url(&value))
                {
                    let _ = element.remove_attribute(&attr_name);
                    continue;
                }

                if tag == "a" && attr_name_lower == "target" {
                    let _ = element.set_attribute("rel", "noopener noreferrer");
                }
            }
        }
    }

    container.inner_html()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_title_safely_and_treats_raw_html_as_text() {
        let html = render_markdown_preview("<Unsafe>", "<script>alert(1)</script>");

        assert!(html.contains("&lt;Unsafe&gt;"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn should_allow_safe_preview_urls() {
        assert!(is_safe_preview_url("https://example.com"));
        assert!(is_safe_preview_url("mailto:test@example.com"));
        assert!(is_safe_preview_url("/notes/123"));
    }

    #[test]
    fn should_reject_unsafe_preview_urls() {
        assert!(!is_safe_preview_url("javascript:alert(1)"));
        assert!(!is_safe_preview_url("data:text/html;base64,AAAA"));
        assert!(!is_safe_preview_url("vbscript:msgbox(1)"));
    }
}
