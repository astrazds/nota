use crate::components::Modal;
use crate::AppState;
use leptos::prelude::*;

#[component]
pub fn ConfirmModal(
    show: RwSignal<bool>,
    title: &'static str,
    message: &'static str,
) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let header = Box::new(move || {
        view! {
            <div>
                <h2 class="text-xl font-bold text-gray-800 dark:text-white">{title}</h2>
                <p class="mt-2 text-gray-500 dark:text-gray-400">{message}</p>
            </div>
        }
        .into_any()
    });

    let footer = Box::new(move || {
        view! {
            <button
                on:click=move |_| show.set(false)
                class="px-5 py-2 font-semibold rounded-lg transition-colors bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20"
            >
                "Cancel"
            </button>
            <button
                on:click=move |_| {
                    state.confirm_delete_selected_note();
                    show.set(false);
                }
                class="px-5 py-2 bg-red-500 text-white font-semibold rounded-lg hover:bg-red-600 transition-colors"
            >
                "Delete"
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
                max_width_class="max-w-sm"
                header=header_clone.clone()
                footer=footer_clone.clone()
            >
                <div class="p-6"></div>
            </Modal>
        </Show>
    }
}
