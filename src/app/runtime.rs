use crate::app::AppState;
use crate::storage::{SaveSession, save_dark_mode, save_sidebar_open};
use leptos::prelude::*;
use nota_core::responsive_navigation::ViewportClass;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, prelude::Closure};

pub fn install_runtime_persistence(state: AppState) -> SaveSession {
    Effect::new(move |_| {
        save_dark_mode(state.is_dark_mode.get());
    });

    Effect::new(move |_| {
        if let Some(stored_state) = state.note_list_state_to_persist() {
            save_sidebar_open(stored_state.is_open());
        }
    });

    let save_session = SaveSession::default();
    let save_session_for_effect = save_session.clone();
    let is_initial_notes_effect = Rc::new(Cell::new(true));
    Effect::new(move |_| {
        let snapshot = state.tracked_notes_persistence_snapshot();
        if is_initial_notes_effect.replace(false) {
            return;
        }
        save_session_for_effect.schedule_collection_save(
            snapshot.notes,
            snapshot.recently_deleted_notes,
            state.save_status,
        );
    });

    save_session.install_page_flush_listeners(
        move || {
            let snapshot = state.persistence_snapshot();
            (snapshot.notes, snapshot.recently_deleted_notes)
        },
        state.save_status,
    );

    save_session
}

pub(super) fn current_viewport_class() -> ViewportClass {
    web_sys::window()
        .and_then(|win| win.inner_width().ok())
        .and_then(|width| width.as_f64())
        .map(ViewportClass::from_width)
        .unwrap_or(ViewportClass::Wide)
}

pub(super) fn install_viewport_listener(state: AppState) {
    let Some(win) = web_sys::window() else {
        return;
    };

    let resize_listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
        let next_viewport_class = current_viewport_class();
        state.reclassify_viewport(next_viewport_class);
    }) as Box<dyn FnMut(_)>);

    let _ =
        win.add_event_listener_with_callback("resize", resize_listener.as_ref().unchecked_ref());
    resize_listener.forget();
}

pub(super) fn install_quick_capture_shortcut(state: AppState) {
    let Some(win) = web_sys::window() else {
        return;
    };

    let keydown_listener = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
        if !is_quick_capture_shortcut(
            &ev.key(),
            ev.ctrl_key(),
            ev.meta_key(),
            ev.alt_key(),
            ev.shift_key(),
        ) {
            return;
        }

        ev.prevent_default();
        state.quick_capture_note();
    }) as Box<dyn FnMut(_)>);

    let _ =
        win.add_event_listener_with_callback("keydown", keydown_listener.as_ref().unchecked_ref());
    keydown_listener.forget();
}

fn is_quick_capture_shortcut(
    key: &str,
    ctrl_key: bool,
    meta_key: bool,
    alt_key: bool,
    shift_key: bool,
) -> bool {
    key.eq_ignore_ascii_case("n") && (ctrl_key || meta_key) && !alt_key && !shift_key
}

#[cfg(test)]
mod shortcut_tests {
    use super::is_quick_capture_shortcut;

    #[test]
    fn quick_capture_shortcut_uses_primary_modifier_and_plain_n() {
        assert!(is_quick_capture_shortcut("n", true, false, false, false));
        assert!(is_quick_capture_shortcut("N", false, true, false, false));

        assert!(!is_quick_capture_shortcut("n", false, false, false, false));
        assert!(!is_quick_capture_shortcut("n", true, false, true, false));
        assert!(!is_quick_capture_shortcut("n", true, false, false, true));
        assert!(!is_quick_capture_shortcut("m", true, false, false, false));
    }
}
