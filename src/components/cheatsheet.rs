use crate::components::Modal;
use crate::AppState;
use leptos::prelude::*;

#[component]
pub fn CheatsheetModal(show: RwSignal<bool>) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let header = Box::new(move || {
        view! {
            <div class="flex justify-between items-center w-full">
                <h2 class=move || {
                    let text_color = if state.is_dark_mode.get() {
                        "text-white"
                    } else {
                        "text-gray-800"
                    };
                    format!("text-2xl font-bold {}", text_color)
                }>"Cheatsheet"</h2>
                <button
                    on:click=move |_| show.set(false)
                    class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
        }
        .into_any()
    });

    let footer = Box::new(move || {
        view! {
            <button
                on:click=move |_| show.set(false)
                class="px-6 py-2 bg-apple-yellow text-white font-semibold rounded-lg hover:bg-yellow-600 transition-colors shadow-md"
            >
                "Got it"
            </button>
        }
        .into_any()
    });

    let header_clone = header.clone();
    let footer_clone = footer.clone();

    view! {
        <Show when=move || show.get()>
            <Modal
                show=show
                header=header_clone.clone()
                footer=footer_clone.clone()
            >
                <div class="p-8 grid grid-cols-1 sm:grid-cols-2 gap-8">
                    <Section
                        title="Headings"
                        items=vec![
                            "# Heading 1",
                            "## Heading 2",
                            "### Heading 3",
                            "#### Heading 4",
                            "##### Heading 5",
                            "###### Heading 6",
                        ]
                    />
                    <Section
                        title="Emphasis"
                        items=vec![
                            "**bold** or __bold__",
                            "*italic* or _italic_",
                            "***bold italic***",
                            "~~strikethrough~~",
                        ]
                    />
                    <Section
                        title="Lists"
                        items=vec![
                            "- Unordered item",
                            "* Also unordered",
                            "1. Ordered item",
                            "   - Nested item",
                        ]
                    />
                    <Section title="Task Lists" items=vec!["- [ ] To do", "- [x] Done"] />
                    <Section
                        title="Links & Images"
                        items=vec![
                            "[Link text](https://example.com)",
                            "<https://example.com>",
                            "![Alt text](https://example.com/image.png)",
                        ]
                    />
                    <Section
                        title="Code"
                        items=vec![
                            "`inline code`",
                            "```rust",
                            "fn main() { println!(\"hi\"); }",
                            "```",
                        ]
                    />
                    <Section
                        title="Quotes & Rules"
                        items=vec!["> Blockquote", "> Nested quote", "--- (horizontal rule)"]
                    />
                    <Section
                        title="Tables"
                        items=vec![
                            "| Name | Value |",
                            "| --- | --- |",
                            "| Foo | Bar |",
                        ]
                    />
                    <Section
                        title="Footnotes"
                        items=vec![
                            "Reference[^1]",
                            "[^1]: Footnote text",
                        ]
                    />
                    <Section
                        title="Line Breaks"
                        items=vec![
                            "End line with two spaces  ",
                            "or use a blank line between paragraphs",
                        ]
                    />
                    <Section
                        title="Escaping"
                        items=vec![
                            "\\*literal asterisks\\*",
                            "\\# literal heading marker",
                        ]
                    />
                    <Section
                        title="Note"
                        items=vec![
                            "Raw HTML is displayed as text for safety.",
                            "Click backdrop or X to close.",
                        ]
                    />
                </div>
            </Modal>
        </Show>
    }
}

#[component]
fn Section(title: &'static str, items: Vec<&'static str>) -> impl IntoView {
    view! {
        <div>
            <h3 class="font-bold text-apple-yellow mb-3 uppercase text-xs tracking-widest">{title}</h3>
            <div class="space-y-2 font-mono text-sm text-gray-600 dark:text-gray-400">
                {items.into_iter().map(|item| view! { <p>{item}</p> }).collect_view()}
            </div>
        </div>
    }
}
