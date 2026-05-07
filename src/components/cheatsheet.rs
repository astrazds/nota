use crate::components::Modal;
use crate::markdown_editing::{MARKDOWN_CHEATSHEET_SECTIONS, MarkdownCheatsheetSection};
use crate::theme::{ThemeAccent, ThemeState, ThemeText};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn CheatsheetModal(show: RwSignal<bool>) -> impl IntoView {
    let close_modal = move || close_markdown_syntax(show);

    let header = Box::new(move || {
        view! {
            <div class="flex w-full items-start justify-between gap-4">
                <div>
                    <h2
                        id="markdown-syntax-title"
                        class=move || format!("text-2xl font-bold leading-tight {}", ThemeText::Primary.classes())
                    >
                        "Markdown syntax"
                    </h2>
                    <p class=move || format!("mt-1 text-sm {}", ThemeText::Muted.classes())>
                        "Syntax Noter renders in Preview."
                    </p>
                </div>
                <button
                    on:click=move |_| close_modal()
                    aria-label="Close markdown syntax"
                    class=move || format!("inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-md transition-colors {}", ThemeState::SidebarToggle.classes())
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
                on:click=move |_| close_modal()
                class=move || format!("min-h-10 rounded-md px-6 py-2 font-semibold transition-colors shadow-sm {}", ThemeAccent::PrimaryFill.classes())
            >
                "Close"
            </button>
        }
        .into_any()
    });

    let header_clone = header.clone();
    let footer_clone = footer.clone();

    view! {
        <Show when=move || show.get()>
            <Modal
                on_dismiss=move || close_modal()
                header=header_clone.clone()
                footer=footer_clone.clone()
                max_width_class="max-w-3xl"
                labelledby="markdown-syntax-title"
            >
                <div class="grid grid-cols-1 gap-6 p-5 sm:grid-cols-[1.15fr_0.85fr] sm:p-6">
                    {section_group(
                        "Core syntax",
                        "The formats most useful while writing Notes.",
                        true,
                    )}
                    {section_group(
                        "Extended syntax",
                        "Special cases and Noter safety behaviour.",
                        false,
                    )}
                </div>
            </Modal>
        </Show>
    }
}

fn close_markdown_syntax(show: RwSignal<bool>) {
    show.set(false);
    focus_markdown_help_button();
}

fn focus_markdown_help_button() {
    let Some(document) = leptos::web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(Some(element)) = document.query_selector("[aria-label='Show markdown cheatsheet']")
    else {
        return;
    };
    let Some(element) = element.dyn_ref::<leptos::web_sys::HtmlElement>() else {
        return;
    };
    let _ = element.focus();
}

fn section_group(title: &'static str, description: &'static str, core: bool) -> impl IntoView {
    view! {
        <section class="min-w-0">
            <h3 class=move || format!("text-sm font-semibold {}", ThemeText::Primary.classes())>{title}</h3>
            <p class=move || format!("mt-1 text-xs {}", ThemeText::Muted.classes())>{description}</p>
            <div class="mt-4 space-y-5">
                {MARKDOWN_CHEATSHEET_SECTIONS
                    .iter()
                    .filter(move |section| is_core_section(section) == core)
                    .map(|section| view! { <Section title=section.title items=section.items /> })
                    .collect_view()}
            </div>
        </section>
    }
}

fn is_core_section(section: &MarkdownCheatsheetSection) -> bool {
    matches!(
        section.title,
        "Headings" | "Emphasis" | "Lists" | "Links & Images" | "Code" | "Quotes & Rules"
    )
}

#[component]
fn Section(title: &'static str, items: &'static [&'static str]) -> impl IntoView {
    view! {
        <div class="min-w-0">
            <h4 class=move || format!("mb-2 text-xs font-semibold {}", ThemeAccent::PrimaryText.classes())>{title}</h4>
            <ul class="space-y-1.5">
                {items
                    .iter()
                    .map(|item| {
                        view! {
                            <li class="min-w-0">
                                <code class=move || format!("block overflow-x-auto rounded-md bg-apple-gray-100 px-2 py-1 font-mono text-[13px] leading-5 dark:bg-white/10 {}", ThemeText::Muted.classes())>
                                    {*item}
                                </code>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}
