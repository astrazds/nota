use crate::app::{AppState, NotificationTone};
use crate::ui::recipes as ui_recipes;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, prelude::Closure};

const NOTIFICATION_HIDE_MS: i32 = 3_000;
type NotificationTimeout = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[component]
pub fn GlobalNotificationOutlet() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let timeout: NotificationTimeout = Rc::new(RefCell::new(None));

    Effect::new(move |_| {
        if let Some((id, _closure)) = timeout.borrow_mut().take()
            && let Some(win) = web_sys::window()
        {
            win.clear_timeout_with_handle(id);
        }

        let Some(notification) = state.notification.get() else {
            return;
        };
        let notification_id = notification.id;
        let timeout_ref = timeout.clone();
        let closure = Closure::wrap(Box::new(move || {
            state.clear_notification(notification_id);
            timeout_ref.borrow_mut().take();
        }) as Box<dyn FnMut()>);

        let Some(win) = web_sys::window() else {
            return;
        };
        if let Ok(id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            NOTIFICATION_HIDE_MS,
        ) {
            *timeout.borrow_mut() = Some((id, closure));
        }
    });

    view! {
        <div class=notification_outlet_classes>
            {move || {
                state.notification.get().map(|notification| {
                    view! {
                        <span
                            role="status"
                            class=notification_classes(notification.tone)
                        >
                            {notification.message}
                        </span>
                    }
                })
            }}
        </div>
    }
}

fn notification_outlet_classes() -> &'static str {
    ui_recipes::global_notification_outlet()
}

fn notification_classes(tone: NotificationTone) -> String {
    ui_recipes::global_notification(match tone {
        NotificationTone::Progress => ui_recipes::GlobalNotificationTone::Progress,
        NotificationTone::Success => ui_recipes::GlobalNotificationTone::Success,
        NotificationTone::Error => ui_recipes::GlobalNotificationTone::Error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_notification_classes_are_visible_but_compact() {
        let outlet = notification_outlet_classes();
        let progress = notification_classes(NotificationTone::Progress);
        let success = notification_classes(NotificationTone::Success);
        let error = notification_classes(NotificationTone::Error);

        assert!(outlet.contains("fixed"));
        assert!(outlet.contains("bottom-16"));
        assert!(outlet.contains("sm:bottom-auto"));
        assert!(outlet.contains("sm:top-5"));
        assert!(outlet.contains("right-3"));
        assert!(outlet.contains("sm:right-5"));
        assert!(outlet.contains("z-50"));
        assert!(outlet.contains("pointer-events-none"));

        for classes in [&progress, &success, &error] {
            assert!(classes.contains("pointer-events-auto"));
            assert!(classes.contains("rounded-md"));
            assert!(classes.contains("border"));
            assert!(classes.contains("shadow-sm"));
            assert!(classes.contains("text-xs"));
            assert!(classes.contains("max-w-[11rem]"));
            assert!(classes.contains("truncate"));
        }

        assert!(progress.contains("bg-apple-yellow/10"));
        assert!(success.contains("bg-emerald-500/10"));
        assert!(error.contains("bg-red-500/10"));
    }
}
