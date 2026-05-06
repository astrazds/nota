use std::collections::HashMap;

use pulldown_cmark::{CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd, html};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct PreviewSafetyPolicy;

impl PreviewSafetyPolicy {
    fn is_safe_url(url: &str) -> bool {
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

    fn is_safe_markdown_url(link_type: LinkType, url: &str) -> bool {
        matches!(link_type, LinkType::Email) || Self::is_safe_url(url)
    }
}

#[cfg(target_arch = "wasm32")]
impl PreviewSafetyPolicy {
    fn should_remove_element(tag: &str, input_type: Option<&str>) -> bool {
        let tag = tag.to_ascii_lowercase();
        matches!(
            tag.as_str(),
            "script" | "style" | "iframe" | "object" | "embed" | "link" | "meta"
        ) || (tag == "input" && input_type != Some("checkbox"))
    }

    fn should_remove_attribute(_tag: &str, attr_name: &str, attr_value: Option<&str>) -> bool {
        let attr_name = attr_name.to_ascii_lowercase();

        attr_name.starts_with("on")
            || attr_name == "style"
            || attr_name == "srcdoc"
            || ((attr_name == "href" || attr_name == "src")
                && attr_value.is_some_and(|value| !Self::is_safe_url(value)))
    }

    fn link_rel_for_target(tag: &str, attr_name: &str) -> Option<&'static str> {
        (tag.eq_ignore_ascii_case("a") && attr_name.eq_ignore_ascii_case("target"))
            .then_some("noopener noreferrer")
    }
}

#[cfg(test)]
pub fn render_markdown_preview(title: &str, content: &str) -> String {
    PreviewPipeline.render(title, content)
}

pub fn render_markdown_preview_body(title: &str, content: &str) -> String {
    PreviewPipeline.render_body(title, content)
}

pub fn markdown_preview_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct PreviewPipeline;

impl PreviewPipeline {
    #[cfg(test)]
    fn render(self, title: &str, content: &str) -> String {
        let safe_title = escape_html(title);
        let body = self.render_body(title, content);

        format!("<h1 class=\"text-3xl font-bold mb-4\">{safe_title}</h1>{body}")
    }

    fn render_body(self, title: &str, content: &str) -> String {
        let events = Parser::new_ext(content, markdown_preview_options())
            .map(escape_user_raw_html)
            .map(neutralize_unsafe_markdown_urls);
        let events = suppress_matching_first_content_h1(title, events);
        let generated_events = render_preview_events(events);
        let generated_html = render_generated_preview_body_html(generated_events);

        sanitize_preview_html(generated_html)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedPreviewHtml(String);

impl GeneratedPreviewHtml {
    #[cfg(target_arch = "wasm32")]
    fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn into_string(self) -> String {
        self.0
    }
}

fn render_generated_preview_body_html<'a>(
    events: impl IntoIterator<Item = Event<'a>>,
) -> GeneratedPreviewHtml {
    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());
    GeneratedPreviewHtml(html_output)
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_user_raw_html(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Html(raw_html) | Event::InlineHtml(raw_html) => Event::Text(raw_html),
        _ => event,
    }
}

fn neutralize_unsafe_markdown_urls(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) if !PreviewSafetyPolicy::is_safe_markdown_url(link_type, &dest_url) => {
            Event::Start(Tag::Link {
                link_type,
                dest_url: "#".into(),
                title,
                id,
            })
        }
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) if !PreviewSafetyPolicy::is_safe_markdown_url(link_type, &dest_url) => {
            Event::Start(Tag::Image {
                link_type,
                dest_url: "#".into(),
                title,
                id,
            })
        }
        _ => event,
    }
}

fn suppress_matching_first_content_h1<'a>(
    title: &str,
    events: impl IntoIterator<Item = Event<'a>>,
) -> Vec<Event<'a>> {
    let events: Vec<_> = events.into_iter().collect();
    let Some(Event::Start(Tag::Heading {
        level: HeadingLevel::H1,
        ..
    })) = events.first()
    else {
        return events;
    };

    let Some(heading_end_index) = events
        .iter()
        .position(|event| matches!(event, Event::End(TagEnd::Heading(HeadingLevel::H1))))
    else {
        return events;
    };

    let heading_text = events[1..heading_end_index]
        .iter()
        .filter_map(|event| match event {
            Event::Text(text) | Event::Code(text) => Some(text.as_ref()),
            _ => None,
        })
        .collect::<String>();

    if heading_text.trim() != title.trim() {
        return events;
    }

    events
        .into_iter()
        .enumerate()
        .filter_map(|(index, event)| (index > heading_end_index).then_some(event))
        .collect()
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
fn sanitize_preview_html(generated_html: GeneratedPreviewHtml) -> String {
    generated_html.into_string()
}

#[cfg(target_arch = "wasm32")]
fn sanitize_preview_html(generated_html: GeneratedPreviewHtml) -> String {
    let raw_html = generated_html.as_str();
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
        let html = render_markdown_preview(
            "<Unsafe>",
            "<a onclick=\"alert(1)\" href=\"javascript:alert(1)\">bad</a>\n\n<script>alert(1)</script>",
        );

        assert!(html.contains("&lt;Unsafe&gt;"));
        assert!(html.contains("&lt;a onclick=\"alert(1)\""));
        assert!(html.contains("href=\"javascript:alert(1)\""));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<a onclick=\"alert(1)\""));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn markdown_preview_keeps_safe_urls_and_rejects_unsafe_markdown_urls() {
        let html = render_markdown_preview(
            "Title",
            "[safe](https://example.com) [mail](mailto:test@example.com) [unsafe](javascript:alert(1)) ![bad](data:text/html;base64,AAAA)",
        );

        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("href=\"mailto:test@example.com\""));
        assert!(!html.contains("javascript:alert"));
        assert!(!html.contains("data:text/html"));
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

    #[test]
    fn markdown_preview_suppresses_matching_first_content_h1() {
        let html = render_markdown_preview(
            "Markdown preview tour",
            "# Markdown preview tour\n\nBody content",
        );

        assert!(html.contains("<h1 class=\"text-3xl font-bold mb-4\">Markdown preview tour</h1>"));
        assert!(!html.contains("<h1>Markdown preview tour</h1>"));
        assert!(html.contains("<p>Body content</p>"));
    }

    #[test]
    fn markdown_preview_body_omits_generated_title_but_keeps_title_suppression() {
        let html = render_markdown_preview_body(
            "Markdown preview tour",
            "# Markdown preview tour\n\nBody content",
        );

        assert!(!html.contains("text-3xl font-bold"));
        assert!(!html.contains("<h1>Markdown preview tour</h1>"));
        assert!(html.contains("<p>Body content</p>"));
    }

    #[test]
    fn markdown_preview_body_keeps_safety_policy_on_the_app_render_path() {
        let html = render_markdown_preview_body(
            "Title",
            "<script>alert(1)</script>\n\n[safe](https://example.com) [unsafe](javascript:alert(1))",
        );

        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("href=\"https://example.com\""));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("javascript:alert"));
    }

    #[test]
    fn markdown_preview_body_preserves_non_matching_content_h1() {
        let html = render_markdown_preview_body("Roadmap", "# Release notes\n\nBody content");

        assert!(!html.contains("text-3xl font-bold"));
        assert!(html.contains("<h1>Release notes</h1>"));
        assert!(html.contains("<p>Body content</p>"));
    }

    #[test]
    fn markdown_preview_preserves_non_matching_first_content_h1() {
        let html = render_markdown_preview("Roadmap", "# Release notes\n\nBody content");

        assert!(html.contains("<h1 class=\"text-3xl font-bold mb-4\">Roadmap</h1>"));
        assert!(html.contains("<h1>Release notes</h1>"));
        assert!(html.contains("<p>Body content</p>"));
    }
}
