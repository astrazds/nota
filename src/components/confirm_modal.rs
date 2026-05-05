use crate::AppState;
use crate::components::Modal;
use leptos::prelude::*;

#[component]
pub fn ConfirmModal(title: &'static str, message: &'static str) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let header = Box::new(move || {
        view! {
            <div>
                <h2 class="text-xl font-bold text-gray-800 dark:text-white">{title}</h2>
                <p class="mt-2 text-gray-500 dark:text-gray-400">
                    {move || {
                        state
                            .delete_confirmation_title()
                            .map(|note_title| format!("\"{note_title}\" will be permanently deleted."))
                            .unwrap_or_else(|| message.to_string())
                    }}
                </p>
            </div>
        }
        .into_any()
    });

    let footer = Box::new(move || {
        view! {
            <button
                on:click=move |_| state.cancel_delete_note()
                class="px-5 py-2 font-semibold rounded-md transition-colors bg-gray-200 text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20"
            >
                "Cancel"
            </button>
            <button
                on:click=move |_| {
                    state.confirm_delete_selected_note();
                }
                class="px-5 py-2 bg-red-500 text-white font-semibold rounded-md hover:bg-red-600 transition-colors focus:outline-none focus:ring-2 focus:ring-red-400"
            >
                "Delete"
            </button>
        }
        .into_any()
    });

    let header_clone = header.clone();
    let footer_clone = footer.clone();

    view! {
        <Show when=move || state.is_delete_confirmation_open()>
            <Modal
                on_dismiss=move || state.cancel_delete_note()
                max_width_class="max-w-sm"
                header=header_clone.clone()
                footer=footer_clone.clone()
            >
                <div class="p-6"></div>
            </Modal>
        </Show>
    }
}
