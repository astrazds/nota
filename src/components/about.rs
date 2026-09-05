use crate::AppState;
use crate::components::Modal;
use crate::theme::{ThemeAccent, ThemeState, ThemeText};
use crate::ui_recipes;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn AboutModal(show: RwSignal<bool>) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let backup_health = Memo::new(move |_| state.backup_health_summary());
    let corrupt_payload_status =
        Memo::new(move |_| about_corrupt_payload_status(state.has_quarantined_corrupt_payloads()));
    let close_modal = move || close_about(show);

    let header = Box::new(move || {
        view! {
            <div class="flex w-full items-start justify-between gap-4">
                <div>
                    <h2
                        id="about-modal-title"
                        class=ui_recipes::modal_title_text
                    >
                        "About Nota"
                    </h2>
                    <p
                        id="about-modal-description"
                        class=ui_recipes::modal_description_text
                    >
                        {about_app_version_label()}
                    </p>
                </div>
                <button
                    on:click=move |_| close_modal()
                    aria-label="Close About Nota"
                    class=move || format!("inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-md transition-colors sm:h-10 sm:w-10 {}", ThemeState::SidebarToggle.classes())
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
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
                data-modal-cancel="true"
                type="button"
                on:click=move |_| close_modal()
                class=move || format!("min-h-11 rounded-md px-6 py-2 transition-colors shadow-sm sm:min-h-10 {} {}", ui_recipes::button_label_text(), ThemeAccent::PrimaryFill.classes())
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
                max_width_class="max-w-md"
                header=header_clone.clone()
                footer=footer_clone.clone()
                labelledby="about-modal-title"
                describedby="about-modal-description"
                initial_focus_selector="[data-modal-cancel='true']"
            >
                <dl class=move || format!("grid gap-4 p-6 sm:grid-cols-[max-content_1fr] {}", ui_recipes::modal_body_text())>
                    <dt class=move || format!("font-medium {}", ThemeText::Muted.classes())>"Storage"</dt>
                    <dd class=move || ThemeText::Primary.classes()>{about_storage_mode_label()}</dd>
                    <dt class=move || format!("font-medium {}", ThemeText::Muted.classes())>"Backup Health"</dt>
                    <dd class=move || ThemeText::Primary.classes()>{move || backup_health.get()}</dd>
                    <dt class=move || format!("font-medium {}", ThemeText::Muted.classes())>"Recovery"</dt>
                    <dd class=move || ThemeText::Primary.classes()>{move || corrupt_payload_status.get()}</dd>
                </dl>
            </Modal>
        </Show>
    }
}

fn close_about(show: RwSignal<bool>) {
    show.set(false);
    focus_about_button();
}

fn focus_about_button() {
    let Some(document) = leptos::web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(Some(element)) = document.query_selector("[aria-label='About Nota']") else {
        return;
    };
    let Some(element) = element.dyn_ref::<leptos::web_sys::HtmlElement>() else {
        return;
    };
    let _ = element.focus();
}

fn about_app_version_label() -> String {
    format!("Nota {}", env!("CARGO_PKG_VERSION"))
}

fn about_storage_mode_label() -> &'static str {
    "Local browser storage"
}

fn about_corrupt_payload_status(has_quarantined_payloads: bool) -> &'static str {
    if has_quarantined_payloads {
        "Corrupt payload quarantined"
    } else {
        "No corrupt payload quarantine"
    }
}

#[cfg(test)]
mod tests {
    use super::{about_app_version_label, about_corrupt_payload_status, about_storage_mode_label};

    #[test]
    fn about_labels_expose_release_storage_and_quarantine_state() {
        assert!(about_app_version_label().starts_with("Nota "));
        assert_eq!(about_storage_mode_label(), "Local browser storage");
        assert_eq!(
            about_corrupt_payload_status(true),
            "Corrupt payload quarantined"
        );
        assert_eq!(
            about_corrupt_payload_status(false),
            "No corrupt payload quarantine"
        );
    }
}
