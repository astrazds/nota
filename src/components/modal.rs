use crate::theme::ThemeSurface;
use leptos::prelude::*;

#[component]
pub fn Modal(
    on_dismiss: impl Fn() + Send + Sync + 'static,
    #[prop(optional)] header: Option<Box<dyn Fn() -> AnyView + Send + Sync + 'static>>,
    #[prop(optional)] footer: Option<Box<dyn Fn() -> AnyView + Send + Sync + 'static>>,
    #[prop(default = "max-w-2xl")] max_width_class: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/35 backdrop-blur-sm transition-opacity duration-200"
            on:click=move |ev| {
                if ev.target() == ev.current_target() {
                    on_dismiss();
                }
            }
            role="dialog"
            aria-modal="true"
        >
            <div class=move || format!("rounded-lg shadow-2xl w-full {} overflow-hidden flex flex-col max-h-[80vh] transform transition-all duration-300 border {}", max_width_class, ThemeSurface::ModalPanel.classes())>
                {header.map(|h| view! {
                    <div class=move || format!("p-6 border-b {}", ThemeSurface::ModalChrome.classes())>
                        {h()}
                    </div>
                })}

                <div class="overflow-y-auto">
                    {children()}
                </div>

                {footer.map(|f| view! {
                    <div class=move || format!("p-4 border-t flex justify-end gap-3 {}", ThemeSurface::ModalChrome.classes())>
                        {f()}
                    </div>
                })}
            </div>
        </div>
    }
}
