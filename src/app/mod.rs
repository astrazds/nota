pub(crate) mod runtime;
mod state;

pub(crate) use state::{AppRuntimeStartup, AppState, NotificationTone};

use self::runtime::{
    current_viewport_class, install_quick_capture_shortcut, install_runtime_persistence,
    install_viewport_listener,
};
use crate::components::{AboutModal, ConfirmModal, Editor, GlobalNotificationOutlet, Sidebar};

use crate::storage::{
    load_backup_health_record, load_collection_startup, load_dark_mode, load_sidebar_open,
};
use crate::ui::theme::ThemeSurface;
use leptos::prelude::*;
use nota_core::responsive_navigation::StoredNoteListState;
use nota_core::storage_recovery::CollectionStartup;

#[component]
pub(super) fn App() -> impl IntoView {
    let collection_startup = load_collection_startup();
    let (notes, recently_deleted_notes, storage_recovery) = match collection_startup {
        CollectionStartup::Ready {
            notes,
            recently_deleted_notes,
        } => (notes, recently_deleted_notes, None),
        CollectionStartup::Recovery(storage_recovery) => {
            (Vec::new(), Vec::new(), Some(storage_recovery))
        }
    };
    let state = AppState::from_startup(AppRuntimeStartup {
        notes,
        recently_deleted_notes,
        is_dark_mode: load_dark_mode(),
        viewport_class: current_viewport_class(),
        stored_note_list_state: StoredNoteListState::from_is_open(load_sidebar_open()),
        backup_health_record: load_backup_health_record(),
        storage_recovery,
    });
    provide_context(state);
    install_viewport_listener(state);
    install_quick_capture_shortcut(state);
    let _save_session = install_runtime_persistence(state);
    let show_about = RwSignal::new(false);

    view! {
        <div
            data-testid="app-frame"
            class=move || {
                let active_theme = if state.is_dark_mode.get() {
                    "bg-apple-notebook-darkFrame text-apple-notebook-frame"
                } else {
                    "bg-apple-notebook-frame text-apple-notebook-graphite"
                };
                format!("{} {active_theme} flex h-screen overflow-hidden p-3", ThemeSurface::RootApp.classes())
            }
            class:dark=move || state.is_dark_mode.get()
        >
            <div
                data-testid="workspace-frame"
                class=move || format!(
                    "flex h-full min-w-0 flex-1 overflow-hidden rounded-lg border shadow-sm {}",
                    ThemeSurface::WorkspaceFrame.classes()
                )
            >
                <Sidebar show_about=show_about />
                <Editor />
            </div>
            <AboutModal show=show_about />
            <GlobalNotificationOutlet />
            <ConfirmModal
                title="Move to Recently Deleted?"
                message="This can be restored from Recently Deleted."
            />
        </div>
    }
}
