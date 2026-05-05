use std::collections::HashMap;

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PreviewSafetyPolicy;

#[cfg(any(test, target_arch = "wasm32"))]
impl PreviewSafetyPolicy {
    pub fn is_safe_url(url: &str) -> bool {
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

    pub fn should_remove_element(tag: &str, input_type: Option<&str>) -> bool {
        let tag = tag.to_ascii_lowercase();
        matches!(
            tag.as_str(),
            "script" | "style" | "iframe" | "object" | "embed" | "link" | "meta"
        ) || (tag == "input" && input_type != Some("checkbox"))
    }

    pub fn should_remove_attribute(_tag: &str, attr_name: &str, attr_value: Option<&str>) -> bool {
        let attr_name = attr_name.to_ascii_lowercase();

        attr_name.starts_with("on")
            || attr_name == "style"
            || attr_name == "srcdoc"
            || ((attr_name == "href" || attr_name == "src")
                && attr_value.is_some_and(|value| !Self::is_safe_url(value)))
    }

    pub fn link_rel_for_target(tag: &str, attr_name: &str) -> Option<&'static str> {
        (tag.eq_ignore_ascii_case("a") && attr_name.eq_ignore_ascii_case("target"))
            .then_some("noopener noreferrer")
    }
}

pub fn render_markdown_preview(title: &str, content: &str) -> String {
    let safe_title = escape_html(title);
    let mut html_output = format!("<h1 class=\"text-3xl font-bold mb-4\">{safe_title}</h1>");
    let parser = Parser::new_ext(content, markdown_preview_options()).map(escape_raw_html_event);
    html::push_html(&mut html_output, render_preview_events(parser).into_iter());
    sanitize_preview_html(&html_output)
}

pub fn markdown_preview_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_raw_html_event(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Html(raw_html) | Event::InlineHtml(raw_html) => Event::Text(raw_html),
        _ => event,
    }
}

fn render_preview_events<'a>(events: impl IntoIterator<Item = Event<'a>>) -> Vec<Event<'a>> {
    let mut preview_events = Vec::new();
    let mut footnote_events = Vec::new();
    let mut current_footnote = Vec::new();
    let mut footnote_numbers: HashMap<CowStr<'a>, (usize, usize)> = HashMap::new();

    for event in events {
        match event {
            Event::Start(Tag::FootnoteDefinition(_)) => {
                current_footnote.push(event);
            }
            Event::End(TagEnd::FootnoteDefinition) if !current_footnote.is_empty() => {
                current_footnote.push(event);
                footnote_events.push(std::mem::take(&mut current_footnote));
            }
            Event::FootnoteReference(name) => {
                let next_number = footnote_numbers.len() + 1;
                let (number, reference_count) = footnote_numbers
                    .entry(name.clone())
                    .or_insert((next_number, 0));
                *reference_count += 1;

                let reference = Event::Html(
                    format!(
                        "<sup class=\"footnote-reference\" id=\"fr-{name}-{reference_count}\"><a href=\"#fn-{name}\">[{number}]</a></sup>",
                        name = escape_html(&name),
                    )
                    .into(),
                );

                if current_footnote.is_empty() {
                    preview_events.push(reference);
                } else {
                    current_footnote.push(reference);
                }
            }
            _ if !current_footnote.is_empty() => {
                current_footnote.push(event);
            }
            _ => {
                preview_events.push(event);
            }
        }
    }

    preview_events.extend(current_footnote);
    append_footnotes(&mut preview_events, footnote_events, &footnote_numbers);
    preview_events
}

fn append_footnotes<'a>(
    preview_events: &mut Vec<Event<'a>>,
    mut footnote_events: Vec<Vec<Event<'a>>>,
    footnote_numbers: &HashMap<CowStr<'a>, (usize, usize)>,
) {
    footnote_events.retain(|events| match events.first() {
        Some(Event::Start(Tag::FootnoteDefinition(name))) => footnote_numbers
            .get(name)
            .is_some_and(|(_, reference_count)| *reference_count > 0),
        _ => false,
    });
    footnote_events.sort_by_key(|events| match events.first() {
        Some(Event::Start(Tag::FootnoteDefinition(name))) => footnote_numbers
            .get(name)
            .map_or(usize::MAX, |(number, _)| *number),
        _ => usize::MAX,
    });

    if footnote_events.is_empty() {
        return;
    }

    preview_events.push(Event::Html("<hr><ol class=\"footnotes-list\">\n".into()));
    for footnote in footnote_events {
        for event in footnote {
            match event {
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    preview_events.push(Event::Html(
                        format!("<li id=\"fn-{}\">", escape_html(&name)).into(),
                    ));
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    preview_events.push(Event::Html("</li>\n".into()));
                }
                _ => preview_events.push(event),
            }
        }
    }
    preview_events.push(Event::Html("</ol>\n".into()));
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
            if PreviewSafetyPolicy::should_remove_element(
                &tag,
                element.get_attribute("type").as_deref(),
            ) {
                element.remove();
                continue;
            }

            let attrs = element.get_attribute_names();
            for attr_index in 0..attrs.length() {
                let Some(attr_name) = attrs.get(attr_index).as_string() else {
                    continue;
                };
                let attr_value = element.get_attribute(&attr_name);

                if PreviewSafetyPolicy::should_remove_attribute(
                    &tag,
                    &attr_name,
                    attr_value.as_deref(),
                ) {
                    let _ = element.remove_attribute(&attr_name);
                    continue;
                }

                if let Some(rel) = PreviewSafetyPolicy::link_rel_for_target(&tag, &attr_name) {
                    let _ = element.set_attribute("rel", rel);
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
        assert!(PreviewSafetyPolicy::is_safe_url("https://example.com"));
        assert!(PreviewSafetyPolicy::is_safe_url("mailto:test@example.com"));
        assert!(PreviewSafetyPolicy::is_safe_url("/notes/123"));
    }

    #[test]
    fn should_reject_unsafe_preview_urls() {
        assert!(!PreviewSafetyPolicy::is_safe_url("javascript:alert(1)"));
        assert!(!PreviewSafetyPolicy::is_safe_url(
            "data:text/html;base64,AAAA"
        ));
        assert!(!PreviewSafetyPolicy::is_safe_url("vbscript:msgbox(1)"));
    }

    #[test]
    fn preview_safety_policy_identifies_unsafe_elements_and_attributes() {
        assert!(PreviewSafetyPolicy::should_remove_element("script", None));
        assert!(PreviewSafetyPolicy::should_remove_element(
            "input",
            Some("text")
        ));
        assert!(!PreviewSafetyPolicy::should_remove_element(
            "input",
            Some("checkbox")
        ));

        assert!(PreviewSafetyPolicy::should_remove_attribute(
            "a", "onclick", None
        ));
        assert!(PreviewSafetyPolicy::should_remove_attribute(
            "a",
            "href",
            Some("javascript:alert(1)")
        ));
        assert!(!PreviewSafetyPolicy::should_remove_attribute(
            "a",
            "href",
            Some("https://example.com")
        ));
        assert_eq!(
            PreviewSafetyPolicy::link_rel_for_target("a", "target"),
            Some("noopener noreferrer")
        );
    }

    #[test]
    fn markdown_preview_supports_documented_dialect() {
        let html = render_markdown_preview(
            "Title",
            "| A | B |\n|---|---|\n| 1 | 2 |\n\n~~done~~\n\n- [ ] task",
        );

        assert!(html.contains("<table>"));
        assert!(html.contains("<del>done</del>"));
        assert!(html.contains("checkbox"));
    }

    #[test]
    fn markdown_preview_renders_usable_footnotes() {
        let html = render_markdown_preview("Title", "Footnote[^1]\n\n[^1]: Footnote text");

        assert!(html.contains("class=\"footnote-reference\""));
        assert!(html.contains("href=\"#fn-1\""));
        assert!(html.contains("<ol class=\"footnotes-list\">"));
        assert!(html.contains("<li id=\"fn-1\">"));
        assert!(html.contains("Footnote text"));
    }
}
