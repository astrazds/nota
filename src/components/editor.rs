use crate::components::CheatsheetModal;
use crate::model::{format_text, parse_tags_input, utf16_range_to_byte_range, Note};
use crate::AppState;
use chrono::Utc;
use leptos::prelude::*;
use pulldown_cmark::{html, Event, Options, Parser};
use wasm_bindgen::JsCast;
use web_sys::window;

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn is_safe_preview_url(url: &str) -> bool {
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

fn tags_to_input(tags: &[String]) -> String {
    tags.join(", ")
}

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let show_cheatsheet = RwSignal::new(false);

    let title_input_ref = NodeRef::<leptos::html::Input>::new();
    let content_area_ref = NodeRef::<leptos::html::Textarea>::new();
    let tags_input_value = RwSignal::new(String::new());
    let is_editing_tags = RwSignal::new(false);

    let selected_note = Memo::new(move |_| {
        let id = state.selected_id.get();
        id.and_then(|id| state.notes.get().iter().find(|n| n.id == id).cloned())
    });

    Effect::new(move |_| {
        if state.focus_title_request.get() {
            if let Some(input) = title_input_ref.get() {
                let _ = input.focus();
            }
            state.focus_title_request.set(false);
        }
    });

    Effect::new(move |_| {
        if is_editing_tags.get() {
            return;
        }

        let tags_value = state
            .selected_id
            .get()
            .and_then(|id| {
                state
                    .notes
                    .get()
                    .iter()
                    .find(|note| note.id == id)
                    .map(|note| tags_to_input(&note.tags))
            })
            .unwrap_or_default();
        tags_input_value.set(tags_value);
    });

    let on_input_content = move |ev| {
        let value = event_target_value(&ev);
        if let Some(id) = state.selected_id.get() {
            state.notes.update(|notes| {
                if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
                    note.content = value.clone();
                    note.last_modified = Utc::now();
                }
            });
        }
    };

    let on_input_title = move |ev| {
        let value = event_target_value(&ev);
        if let Some(id) = state.selected_id.get() {
            state.notes.update(|notes| {
                if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
                    note.title = value.clone();
                    note.last_modified = Utc::now();
                }
            });
        }
    };

    let on_input_tags = move |ev| {
        let value = event_target_value(&ev);
        tags_input_value.set(value.clone());

        let parsed_tags = parse_tags_input(&value);
        if let Some(id) = state.selected_id.get() {
            state.notes.update(|notes| {
                if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
                    if note.tags != parsed_tags {
                        note.tags = parsed_tags.clone();
                        note.last_modified = Utc::now();
                    }
                }
            });
        }
    };

    let commit_tags_input = move || {
        let parsed_tags = parse_tags_input(&tags_input_value.get_untracked());
        if let Some(id) = state.selected_id.get_untracked() {
            let normalised_input = tags_to_input(&parsed_tags);
            state.notes.update(|notes| {
                if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
                    if note.tags != parsed_tags {
                        note.tags = parsed_tags.clone();
                        note.last_modified = Utc::now();
                    }
                }
            });
            tags_input_value.set(normalised_input);
        }
    };

    let markdown_html = Memo::new(move |_| {
        let note = selected_note.get();
        let title = note
            .as_ref()
            .map(|n: &Note| n.display_title().to_string())
            .unwrap_or_default();
        let content = note
            .as_ref()
            .map(|n: &Note| n.content.as_str())
            .unwrap_or_default();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        let safe_title = escape_html(&title);
        let mut html_output = format!("<h1 class=\"text-3xl font-bold mb-4\">{safe_title}</h1>");
        let parser = Parser::new_ext(content, options).map(|event| match event {
            Event::Html(raw_html) | Event::InlineHtml(raw_html) => Event::Text(raw_html),
            _ => event,
        });
        html::push_html(&mut html_output, parser);
        sanitize_preview_html(&html_output)
    });

    let apply_format = move |prefix: &str, suffix: &str| {
        if let Some(textarea) = content_area_ref.get() {
            let start_utf16 = textarea.selection_start().unwrap_or_default().unwrap_or(0);
            let end_utf16 = textarea.selection_end().unwrap_or_default().unwrap_or(0);
            let (selection_start_utf16, selection_end_utf16) = if start_utf16 <= end_utf16 {
                (start_utf16, end_utf16)
            } else {
                (end_utf16, start_utf16)
            };
            let content = textarea.value();
            let (start, end) = utf16_range_to_byte_range(
                &content,
                selection_start_utf16 as usize,
                selection_end_utf16 as usize,
            );

            let new_content = format_text(&content, start, end, prefix, suffix);

            // Use the selected_note memo to get the current note ID without re-searching
            if let Some(note) = selected_note.get_untracked() {
                state.notes.update(|notes| {
                    if let Some(n) = notes.iter_mut().find(|n| n.id == note.id) {
                        n.content = new_content;
                        n.last_modified = Utc::now();
                    }
                });
            }

            let _ = textarea.focus();
            let new_cursor_pos = selection_start_utf16
                + prefix.encode_utf16().count() as u32
                + (selection_end_utf16 - selection_start_utf16)
                + suffix.encode_utf16().count() as u32;
            let _ = textarea.set_selection_start(Some(new_cursor_pos));
            let _ = textarea.set_selection_end(Some(new_cursor_pos));
        }
    };

    view! {
        <div class="flex-1 flex flex-col h-full overflow-hidden relative transition-colors duration-300">
            <CheatsheetModal show=show_cheatsheet />

            <Show when=move || !state.is_sidebar_open.get()>
                <button
                    on:click=move |_| state.is_sidebar_open.set(true)
                    class="absolute left-0 top-1/2 -translate-y-1/2 z-20 p-2 bg-apple-yellow text-white rounded-r-lg shadow-lg hover:bg-yellow-600 transition-colors"
                    title="Expand sidebar"
                    aria-label="Expand sidebar"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
                    </svg>
                </button>
            </Show>

            <div class="p-2 px-4 flex justify-between items-center border-b sticky top-0 z-10 transition-colors bg-white border-apple-gray-200 dark:bg-apple-dark-bg dark:border-apple-dark-border">
                <div class="flex items-center space-x-2">
                    <button
                        on:click=move |_| state.is_sidebar_open.update(|v| *v = !*v)
                        class="p-2 lg:hidden text-gray-500 hover:text-apple-yellow transition-colors"
                        title="Toggle Sidebar"
                        aria-label="Toggle sidebar"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                        </svg>
                    </button>
                    <div class="flex space-x-1 border-r pr-4 border-gray-200 dark:border-apple-dark-border">
                        <button
                            on:click=move |_| state.show_preview.update(|v| *v = !*v)
                            title="Toggle Preview"
                            aria-label=move || if state.show_preview.get() { "Hide preview" } else { "Show preview" }
                            class=move || {
                                if state.show_preview.get() {
                                    "px-3 py-1 text-sm rounded-md transition-all border bg-apple-yellow/10 border-apple-yellow text-apple-yellow"
                                } else {
                                    "px-3 py-1 text-sm rounded-md transition-all border bg-white border-gray-200 text-gray-500 hover:border-gray-300 dark:bg-white/5 dark:border-apple-dark-border dark:text-gray-400 dark:hover:border-gray-500"
                                }
                            }
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                            </svg>
                        </button>
                        <button
                            on:click=move |_| show_cheatsheet.set(true)
                            title="Markdown Help"
                            aria-label="Show markdown cheatsheet"
                            class="px-3 py-1 text-sm rounded-md border transition-colors border-gray-200 bg-white text-gray-500 hover:bg-gray-50 dark:border-apple-dark-border dark:bg-white/5 dark:text-gray-400 dark:hover:bg-white/10"
                        >
                            "?"
                        </button>
                    </div>

                    <div class="flex items-center space-x-1">
                        <ToolbarButton on_click=move |_| apply_format("**", "**") title="Bold" aria_label="Bold">
                            <span class="font-bold">B</span>
                        </ToolbarButton>
                        <ToolbarButton on_click=move |_| apply_format("*", "*") title="Italic" aria_label="Italic">
                            <span class="italic">I</span>
                        </ToolbarButton>
                        <ToolbarButton on_click=move |_| apply_format("~~", "~~") title="Strikethrough" aria_label="Strikethrough">
                            <span class="line-through">S</span>
                        </ToolbarButton>
                        <ToolbarButton on_click=move |_| apply_format("- [ ] ", "") title="Task List" aria_label="Task list">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h4v4H4V6zm0 8h4v4H4v-4zm0 8h4v-4H4v4zM12 7h8M12 15h8M12 19h8" />
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 7.5l1.2 1.2L7.8 7" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarButton
                            on_click=move |_| {
                                apply_format(
                                    "\n| Column 1 | Column 2 |\n| --- | --- |\n| Value 1 | Value 2 |\n",
                                    "",
                                )
                            }
                            title="Insert Table"
                            aria_label="Insert table"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6h18v12H3V6zM3 12h18M9 6v12M15 6v12" />
                            </svg>
                        </ToolbarButton>
                    </div>
                </div>
            </div>

            <div class="flex-1 flex overflow-hidden">
                {move || match selected_note.get() {
                    Some(note) => view! {
                        <div class="flex-1 flex overflow-hidden divide-x divide-apple-gray-200 dark:divide-apple-dark-border">
                            <div class="flex-1 flex flex-col overflow-hidden bg-white dark:bg-apple-dark-bg">
                                <input
                                    node_ref=title_input_ref
                                    type="text"
                                    class="p-8 pb-0 text-3xl font-bold focus:outline-none bg-transparent placeholder:text-gray-300 dark:placeholder:text-gray-600 dark:text-white"
                                    placeholder="Note Title"
                                    prop:value=note.title
                                    on:input=on_input_title
                                />
                                <textarea
                                    node_ref=content_area_ref
                                    class="flex-1 p-8 pt-4 text-lg focus:outline-none resize-none bg-transparent selection:bg-apple-yellow/30 font-mono dark:text-gray-300"
                                    placeholder="Start typing..."
                                    prop:value=note.content
                                    on:input=on_input_content
                                ></textarea>
                                <div class="h-16 px-8 border-t border-apple-gray-200 dark:border-apple-dark-border flex items-center">
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 text-sm rounded-md focus:outline-none transition-colors bg-black/5 text-gray-700 placeholder-gray-400 focus:bg-black/10 dark:bg-white/10 dark:text-gray-200 dark:placeholder-gray-500 dark:focus:bg-white/20"
                                        placeholder="Tags (comma separated)"
                                        prop:value=move || tags_input_value.get()
                                        on:focus=move |_| is_editing_tags.set(true)
                                        on:input=on_input_tags
                                        on:blur=move |_| {
                                            is_editing_tags.set(false);
                                            commit_tags_input();
                                        }
                                    />
                                </div>
                            </div>

                            <Show when=move || state.show_preview.get()>
                                <div class="hidden md:block flex-1 p-8 overflow-y-auto prose max-w-none break-words shadow-inner border-l transition-colors bg-gray-50 prose-yellow border-apple-gray-200 dark:bg-white/5 dark:prose-invert dark:border-apple-dark-border">
                                    <div inner_html=markdown_html.get()></div>
                                </div>
                            </Show>
                        </div>
                    }.into_any(),
                    None => view! {
                        <div class="flex-1 flex items-center justify-center text-gray-300 dark:text-gray-700 select-none">
                            <div class="text-center">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-20 w-20 mx-auto opacity-20 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                </svg>
                                <p class="text-xl">No Note Selected</p>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn ToolbarButton(
    on_click: impl Fn(leptos::web_sys::MouseEvent) + Send + Sync + 'static,
    title: &'static str,
    aria_label: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            on:click=on_click
            title=title
            aria-label=aria_label
            class="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded text-gray-600 dark:text-gray-400"
        >
            {children()}
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::is_safe_preview_url;

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
