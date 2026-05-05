use leptos::prelude::*;

#[component]
pub fn Modal(
    show: RwSignal<bool>,
    #[prop(optional)] header: Option<Box<dyn Fn() -> AnyView + Send + Sync + 'static>>,
    #[prop(optional)] footer: Option<Box<dyn Fn() -> AnyView + Send + Sync + 'static>>,
    #[prop(default = "max-w-2xl")] max_width_class: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/40 backdrop-blur-sm transition-opacity duration-200"
            on:click=move |ev| {
                if ev.target() == ev.current_target() {
                    show.set(false);
                }
            }
            role="dialog"
            aria-modal="true"
        >
            <div class=move || format!("rounded-2xl shadow-2xl w-full {} overflow-hidden flex flex-col max-h-[80vh] transform transition-all duration-300 bg-white dark:bg-apple-dark-sidebar dark:border dark:border-apple-dark-border", max_width_class)>
                {header.map(|h| view! {
                    <div class="p-6 border-b bg-gray-50 border-gray-100 dark:bg-white/5 dark:border-apple-dark-border">
                        {h()}
                    </div>
                })}

                <div class="overflow-y-auto">
                    {children()}
                </div>

                {footer.map(|f| view! {
                    <div class="p-4 border-t flex justify-end gap-3 bg-gray-50 border-gray-100 dark:bg-white/5 dark:border-apple-dark-border">
                        {f()}
                    </div>
                })}
            </div>
        </div>
    }
}
