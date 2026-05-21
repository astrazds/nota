use crate::theme::ThemeSurface;
use leptos::prelude::*;
use std::sync::Arc;
use wasm_bindgen::JsCast;

#[component]
pub fn Modal(
    on_dismiss: impl Fn() + Send + Sync + 'static,
    #[prop(optional)] header: Option<Box<dyn Fn() -> AnyView + Send + Sync + 'static>>,
    #[prop(optional)] footer: Option<Box<dyn Fn() -> AnyView + Send + Sync + 'static>>,
    #[prop(default = "max-w-2xl")] max_width_class: &'static str,
    #[prop(optional)] labelledby: Option<&'static str>,
    #[prop(optional)] describedby: Option<&'static str>,
    #[prop(optional)] initial_focus_selector: Option<&'static str>,
    #[prop(default = false)] hide_body: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let dialog_panel_ref = NodeRef::<leptos::html::Div>::new();
    let restore_focus = leptos::web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
        .and_then(|element| element.dyn_into::<leptos::web_sys::HtmlElement>().ok());
    let on_dismiss = Arc::new(on_dismiss);
    let dismiss_on_click = Arc::clone(&on_dismiss);
    let dismiss_on_keydown = Arc::clone(&on_dismiss);
    let restore_on_click = restore_focus.clone();
    let restore_on_keydown = restore_focus.clone();

    Effect::new(move |_| {
        if let Some(dialog) = dialog_panel_ref.get() {
            let focused = initial_focus_selector
                .and_then(|selector| dialog.query_selector(selector).ok().flatten())
                .and_then(|element| element.dyn_into::<leptos::web_sys::HtmlElement>().ok());
            if let Some(element) = focused {
                let _ = element.focus();
            } else {
                let _ = dialog.focus();
            }
        }
    });

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-apple-notebook-graphite/35 p-4 transition-opacity duration-200 dark:bg-apple-notebook-darkFrame/65"
            on:click=move |ev| {
                if ev.target() == ev.current_target() {
                    (*dismiss_on_click)();
                    restore_previous_focus(&restore_on_click);
                }
            }
            on:keydown=move |ev: leptos::web_sys::KeyboardEvent| {
                if ev.key() == "Escape" {
                    ev.prevent_default();
                    (*dismiss_on_keydown)();
                    restore_previous_focus(&restore_on_keydown);
                }
            }
        >
            <div
                node_ref=dialog_panel_ref
                tabindex="-1"
                role="dialog"
                aria-modal="true"
                aria-labelledby=labelledby
                aria-describedby=describedby
                class=move || format!("w-full {} max-h-[82vh] overflow-hidden rounded-md border shadow-xl transform transition-all duration-200 flex flex-col {}", max_width_class, ThemeSurface::ModalPanel.classes())
            >
                {header.map(|h| view! {
                    <div class=move || format!("border-b p-5 {}", ThemeSurface::ModalChrome.classes())>
                        {h()}
                    </div>
                })}

                <Show when=move || should_render_modal_body(hide_body)>
                    <div class="overflow-y-auto">
                        {children()}
                    </div>
                </Show>

                {footer.map(|f| view! {
                    <div class=move || format!("flex justify-end gap-2 border-t p-3 {}", ThemeSurface::ModalChrome.classes())>
                        {f()}
                    </div>
                })}
            </div>
        </div>
    }
}

fn should_render_modal_body(hide_body: bool) -> bool {
    !hide_body
}

fn restore_previous_focus(element: &Option<leptos::web_sys::HtmlElement>) {
    if let Some(element) = element {
        let _ = element.focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_can_hide_empty_body_region() {
        assert!(!should_render_modal_body(true));
        assert!(should_render_modal_body(false));
    }
}
