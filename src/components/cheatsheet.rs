use crate::components::Modal;
use crate::markdown_editing::MARKDOWN_CHEATSHEET_SECTIONS;
use crate::theme::{ThemeAccent, ThemeState, ThemeText};
use leptos::prelude::*;

#[component]
pub fn CheatsheetModal(show: RwSignal<bool>) -> impl IntoView {
    let header = Box::new(move || {
        view! {
            <div class="flex justify-between items-center w-full">
                <h2 class=move || format!("text-2xl font-bold {}", ThemeText::Primary.classes())>"Markdown help"</h2>
                <button
                    on:click=move |_| show.set(false)
                    aria-label="Close markdown help"
                    class=move || ThemeState::SidebarToggle.classes()
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
                class=move || format!("px-6 py-2 font-semibold rounded-md transition-colors shadow-sm {}", ThemeAccent::PrimaryFill.classes())
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
                on_dismiss=move || show.set(false)
                header=header_clone.clone()
                footer=footer_clone.clone()
            >
                <div class="p-8 grid grid-cols-1 sm:grid-cols-2 gap-8">
                    {MARKDOWN_CHEATSHEET_SECTIONS
                        .iter()
                        .map(|section| view! { <Section title=section.title items=section.items /> })
                        .collect_view()}
                </div>
            </Modal>
        </Show>
    }
}

#[component]
fn Section(title: &'static str, items: &'static [&'static str]) -> impl IntoView {
    view! {
        <div>
            <h3 class=move || format!("font-bold mb-3 uppercase text-xs tracking-widest {}", ThemeAccent::PrimaryText.classes())>{title}</h3>
            <div class=move || format!("space-y-2 font-mono text-sm {}", ThemeText::Muted.classes())>
                {items.iter().map(|item| view! { <p>{*item}</p> }).collect_view()}
            </div>
        </div>
    }
}
