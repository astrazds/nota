use crate::AppState;
use crate::components::Modal;
use crate::theme::{ThemeState, ThemeText};
use leptos::prelude::*;

#[component]
pub fn ConfirmModal(title: &'static str, message: &'static str) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let is_confirmation_open = move || {
        state.is_delete_confirmation_open()
            || state
                .clear_all_recently_deleted_confirmation_count()
                .is_some()
    };
    let dismiss_confirmation = move || {
        if state
            .clear_all_recently_deleted_confirmation_count()
            .is_some()
        {
            state.cancel_clear_all_recently_deleted();
        } else {
            state.cancel_delete_note();
        }
    };

    let header = Box::new(move || {
        view! {
            <div>
                <h2 class=move || format!("text-xl font-bold {}", ThemeText::Primary.classes())>
                    {move || {
                        if state
                            .clear_all_recently_deleted_confirmation_count()
                            .is_some()
                        {
                            "Clear Recently Deleted?"
                        } else {
                            title
                        }
                    }}
                </h2>
                <p class=move || format!("mt-2 {}", ThemeText::Muted.classes())>
                    {move || {
                        if let Some(count) = state.clear_all_recently_deleted_confirmation_count() {
                            clear_all_recently_deleted_confirmation_message(count)
                        } else {
                            state
                                .delete_confirmation_title()
                                .map(delete_confirmation_message)
                                .unwrap_or_else(|| message.to_string())
                        }
                    }}
                </p>
            </div>
        }
        .into_any()
    });

    let footer = Box::new(move || {
        view! {
            <button
                on:click=move |_| dismiss_confirmation()
                class=move || format!("px-5 py-2 font-semibold rounded-md transition-colors {}", ThemeState::SecondaryButton.classes())
            >
                "Cancel"
            </button>
            <button
                on:click=move |_| {
                    if state
                        .clear_all_recently_deleted_confirmation_count()
                        .is_some()
                    {
                        state.confirm_clear_all_recently_deleted_notes();
                    } else {
                        state.confirm_delete_selected_note();
                    }
                }
                class=move || format!("px-5 py-2 font-semibold rounded-md transition-colors {}", ThemeState::DangerButton.classes())
            >
                {move || {
                    if state
                        .clear_all_recently_deleted_confirmation_count()
                        .is_some()
                    {
                        "Clear All"
                    } else {
                        "Delete"
                    }
                }}
            </button>
        }
        .into_any()
    });

    let header_clone = header.clone();
    let footer_clone = footer.clone();

    view! {
        <Show when=is_confirmation_open>
            <Modal
                on_dismiss=dismiss_confirmation
                max_width_class="max-w-sm"
                header=header_clone.clone()
                footer=footer_clone.clone()
                hide_body=true
            >
                <div></div>
            </Modal>
        </Show>
    }
}

fn delete_confirmation_message(note_title: String) -> String {
    format!("\"{note_title}\" will move to Recently Deleted.")
}

fn clear_all_recently_deleted_confirmation_message(count: usize) -> String {
    let note_label = if count == 1 { "Note" } else { "Notes" };
    format!("This will permanently clear {count} recently deleted {note_label}.")
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

    #[test]
    fn clear_all_confirmation_copy_names_the_affected_note_count() {
        assert_eq!(
            clear_all_recently_deleted_confirmation_message(1),
            "This will permanently clear 1 recently deleted Note."
        );
        assert_eq!(
            clear_all_recently_deleted_confirmation_message(3),
            "This will permanently clear 3 recently deleted Notes."
        );
    }
}
