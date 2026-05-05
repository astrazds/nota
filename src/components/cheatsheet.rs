use crate::AppState;
use crate::components::Modal;
use crate::markdown_editing::MARKDOWN_CHEATSHEET_SECTIONS;
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
            <h3 class="font-bold text-apple-yellow mb-3 uppercase text-xs tracking-widest">{title}</h3>
            <div class="space-y-2 font-mono text-sm text-gray-600 dark:text-gray-400">
                {items.iter().map(|item| view! { <p>{*item}</p> }).collect_view()}
            </div>
        </div>
    }
}
