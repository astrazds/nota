use crate::AppState;
use crate::components::Modal;
use crate::theme::{ThemeState, ThemeText};
use leptos::prelude::*;

#[component]
pub fn ConfirmModal(title: &'static str, message: &'static str) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let header = Box::new(move || {
        view! {
            <div>
                <h2 class=move || format!("text-xl font-bold {}", ThemeText::Primary.classes())>{title}</h2>
                <p class=move || format!("mt-2 {}", ThemeText::Muted.classes())>
                    {move || {
                        state
                            .delete_confirmation_title()
                            .map(delete_confirmation_message)
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
                class=move || format!("px-5 py-2 font-semibold rounded-md transition-colors {}", ThemeState::SecondaryButton.classes())
            >
                "Cancel"
            </button>
            <button
                on:click=move |_| {
                    state.confirm_delete_selected_note();
                }
                class=move || format!("px-5 py-2 font-semibold rounded-md transition-colors {}", ThemeState::DangerButton.classes())
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

fn delete_confirmation_message(note_title: String) -> String {
    format!("\"{note_title}\" will move to Recently Deleted.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_confirmation_copy_matches_recoverable_delete_behavior() {
        assert_eq!(
            delete_confirmation_message("Draft".to_string()),
            "\"Draft\" will move to Recently Deleted."
        );
    }
}
