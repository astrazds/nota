use crate::AppState;
use crate::backup_controls::SidebarBackupControls;
use crate::components::Modal;
use crate::note_discovery::NoteListItem;
use crate::note_list_interaction::{NoteListDisplayState, SEARCH_DEBOUNCE_MS};
use crate::theme::{ThemeAccent, ThemeState, ThemeSurface, ThemeText};
use crate::ui_recipes;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::window;

const SIDEBAR_BASE_CLASS: &str = "fixed inset-y-0 left-0 z-30 transform transition-all duration-300 ease-in-out lg:relative lg:translate-x-0 flex flex-col h-full border-r";
type TimeoutState = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

fn sidebar_state_class(is_open: bool) -> &'static str {
    if is_open {
        "translate-x-0 w-full max-w-full lg:w-80"
    } else {
        "-translate-x-full w-0 overflow-hidden"
    }
}

fn search_syntax_hint_classes() -> String {
    ui_recipes::search_hint()
}

fn search_input_classes() -> String {
    ui_recipes::search_input()
}

fn search_clear_button_classes() -> String {
    ui_recipes::search_clear_button()
}

fn search_syntax_hint_code_classes() -> String {
    ui_recipes::search_hint_code()
}

fn sidebar_title_classes() -> String {
    ui_recipes::app_title_text()
}

fn app_version_label() -> String {
    format!("Noter {}", env!("CARGO_PKG_VERSION"))
}

fn storage_mode_label() -> &'static str {
    "Local browser storage"
}

fn corrupt_payload_status(has_quarantined_payloads: bool) -> &'static str {
    if has_quarantined_payloads {
        "Corrupt payload quarantined"
    } else {
        "No corrupt payload quarantine"
    }
}

fn recovery_action_button_classes() -> String {
    ui_recipes::recovery_action_button()
}

fn recovery_danger_button_classes() -> String {
    ui_recipes::recovery_danger_button()
}

fn recently_deleted_clear_all_button_classes() -> String {
    ui_recipes::danger_footer_button()
}

fn recently_deleted_permanent_delete_label() -> &'static str {
    "Delete forever"
}

fn should_show_clear_all_recently_deleted(count: usize) -> bool {
    count > 0
}

fn note_list_title_classes() -> String {
    ui_recipes::note_list_title_text()
}

fn note_list_result_status_classes() -> String {
    format!("text-xs leading-5 {}", ThemeText::Muted.classes())
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let add_note = move |_| state.create_note();
    let show_diagnostics = RwSignal::new(false);

    let note_projection = Memo::new(move |_| state.note_list_projection());
    let note_result_status = Memo::new(move |_| {
        let projection = note_projection.get();
        state.note_list_result_status(&projection)
    });
    let filtered_empty_message = Memo::new(move |_| state.note_list_filtered_empty_message());
    let available_tags = Memo::new(move |_| state.available_tags());
    let recently_deleted_notes = Memo::new(move |_| state.recently_deleted_notes());
    let diagnostics_backup_health = Memo::new(move |_| state.backup_health_summary());
    let diagnostics_corrupt_payload_status =
        Memo::new(move |_| corrupt_payload_status(state.has_quarantined_corrupt_payloads()));

    let search_input_value = RwSignal::new(state.note_search_input());
    let search_hint_open = RwSignal::new(false);
    let debounce_timeout: TimeoutState = Rc::new(RefCell::new(None));
    let clear_search = move |_| {
        state.edit_note_search(String::new());
        state.commit_note_search();
        search_input_value.set(String::new());
    };

    Effect::new(move |_| {
        let _input = search_input_value.get();
        let timeout_ref = debounce_timeout.clone();

        if let Some((id, _)) = timeout_ref.borrow_mut().take()
            && let Some(win) = window()
        {
            win.clear_timeout_with_handle(id);
        }

        if let Some(win) = window() {
            let closure = Closure::wrap(Box::new(move || {
                state.commit_note_search();
            }) as Box<dyn FnMut()>);

            let id = win
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    SEARCH_DEBOUNCE_MS,
                )
                .ok();

            if let Some(id) = id {
                *timeout_ref.borrow_mut() = Some((id, closure));
            }
        }
    });

    view! {
        <div
            class=move || {
                let state_class = sidebar_state_class(state.is_sidebar_open.get());
                format!("{SIDEBAR_BASE_CLASS} {} {state_class}", ThemeSurface::Sidebar.classes())
            }
            role="navigation"
            aria-label="Notes sidebar"
        >
            <Show when=move || state.is_sidebar_open.get()>
                <div class="p-4 space-y-4 sticky top-0 z-10">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center space-x-2">
                            <button
                                on:click=move |_| state.toggle_dark_mode()
                                class=move || format!("p-2 rounded-full {}", ThemeState::IconButton.classes())
                                title="Toggle Theme"
                                aria-label=move || if state.is_dark_mode.get() { "Switch to light mode" } else { "Switch to dark mode" }
                            >
                                {move || if state.is_dark_mode.get() {
                                    view! { <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20"><path d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.464 5.05l-.707-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" /></svg> }.into_any()
                                } else {
                                    view! { <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-500" fill="currentColor" viewBox="0 0 20 20"><path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" /></svg> }.into_any()
                                }}
                            </button>
                            <h1 class=sidebar_title_classes>"Notes"</h1>
                        </div>
                        <div class="flex items-center space-x-1">
                            <button
                                on:click=move |_| state.toggle_sidebar()
                                class=move || format!("p-2 lg:hidden {}", ThemeState::SidebarToggle.classes())
                                title=move || if state.is_sidebar_open.get() { "Collapse sidebar" } else { "Expand sidebar" }
                                aria-label=move || if state.is_sidebar_open.get() { "Collapse sidebar" } else { "Expand sidebar" }
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
                                </svg>
                            </button>
                            <button
                                on:click=move |_| show_diagnostics.set(true)
                                title="About Noter"
                                class=move || format!("p-2 rounded-full {}", ThemeState::IconButton.classes())
                                aria-label="About Noter"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 17h.01M12 13v-3m0 11a9 9 0 100-18 9 9 0 000 18z" />
                                </svg>
                            </button>
                            <button
                                on:click=add_note
                                title="New Note"
                                class=move || format!("inline-flex items-center gap-1 rounded-md px-2 py-1 text-sm font-medium transition-colors {}", ThemeAccent::PrimaryText.classes())
                                aria-label="Create new note"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                </svg>
                                <span class="hidden sm:inline">"New"</span>
                            </button>
                        </div>
                    </div>

                    <div class="relative group">
                        <span class="absolute inset-y-0 left-0 pl-3 flex items-center text-gray-400">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                        </span>
                        <input
                            type="text"
                            placeholder="Search"
                            aria-label="Search notes"
                            aria-describedby="search-syntax-hint"
                            class=search_input_classes
                            prop:value=move || search_input_value.get()
                            on:focus=move |_| search_hint_open.set(true)
                            on:blur=move |_| search_hint_open.set(false)
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                state.edit_note_search(value.clone());
                                search_input_value.set(value);
                            }
                        />
                        <Show when=move || !search_input_value.get().is_empty()>
                            <button
                                type="button"
                                class=search_clear_button_classes
                                title="Clear search"
                                aria-label="Clear search"
                                on:click=clear_search
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                    <path fill-rule="evenodd" d="M10 8.586l3.293-3.293a1 1 0 111.414 1.414L11.414 10l3.293 3.293a1 1 0 01-1.414 1.414L10 11.414l-3.293 3.293a1 1 0 01-1.414-1.414L8.586 10 5.293 6.707A1 1 0 016.707 5.293L10 8.586z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </Show>
                        <Show when=move || search_hint_open.get()>
                            <div
                                id="search-syntax-hint"
                                class=search_syntax_hint_classes
                                role="status"
                            >
                                <div class="mb-1 font-medium">"Syntax"</div>
                                <div class="flex flex-wrap gap-1" aria-label="Supported search syntax">
                                    <code class=search_syntax_hint_code_classes>"\"phrase\""</code>
                                    <code class=search_syntax_hint_code_classes>"title:plan"</code>
                                    <code class=search_syntax_hint_code_classes>"tag:work"</code>
                                    <code class=search_syntax_hint_code_classes>"is:pinned"</code>
                                </div>
                            </div>
                        </Show>
                    </div>

                    <Show when=move || state.active_tag().is_some() && !available_tags.get().is_empty()>
                        <div class="flex items-center gap-2 text-xs">
                            <span class=move || ThemeText::Muted.classes()>"Filtered by"</span>
                            <button
                                class=move || format!("px-2 py-0.5 rounded-full {}", ThemeState::FilterPill.classes())
                                on:click=move |_| state.clear_active_tag()
                                title="Clear tag filter"
                                aria-label="Clear tag filter"
                            >
                                {move || state.active_tag().map(|tag| format!("#{tag}")).unwrap_or_else(|| "All".to_string())}
                            </button>
                        </div>
                    </Show>
                    <Show when=move || note_result_status.get().is_some()>
                        <p
                            class=note_list_result_status_classes
                            aria-live="polite"
                        >
                            {move || {
                                note_result_status
                                    .get()
                                    .map(|status| status.text)
                                    .unwrap_or_default()
                            }}
                        </p>
                    </Show>
                </div>
                <div class="flex-1 overflow-y-auto pb-4">
                    <Show when=move || !note_projection.get().rows.is_empty()>
                        <For
                            each=move || note_projection.get().rows
                            key=|item| item.render_key()
                            let:item
                        >
                            <NoteItem item=item />
                        </For>
                    </Show>
                    <Show when=move || {
                        let projection = note_projection.get();
                        state.note_list_display_state(&projection) == NoteListDisplayState::FilteredEmpty
                    }>
                        <div class=move || format!("p-8 text-center {}", ThemeText::Subtle.classes())>
                            <p>{move || filtered_empty_message.get().title}</p>
                            <p class="text-sm mt-1">{move || filtered_empty_message.get().body}</p>
                        </div>
                    </Show>
                    <Show when=move || !recently_deleted_notes.get().is_empty()>
                        <details class="mx-4 mt-4 rounded-md border border-apple-gray-300 text-sm dark:border-apple-dark-border">
                            <summary class=move || format!("flex cursor-pointer items-center gap-2 px-3 py-2 font-medium {}", ThemeText::Primary.classes())>
                                <span>{move || format!("Recently Deleted ({})", recently_deleted_notes.get().len())}</span>
                                <Show when=move || should_show_clear_all_recently_deleted(recently_deleted_notes.get().len())>
                                    <button
                                        type="button"
                                        class=recently_deleted_clear_all_button_classes
                                        data-confirm-return="clear-all"
                                        on:click=move |ev: leptos::web_sys::MouseEvent| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            state.request_clear_all_recently_deleted();
                                        }
                                    >
                                        "Clear All"
                                    </button>
                                </Show>
                            </summary>
                            <div class="divide-y divide-apple-gray-200 dark:divide-apple-dark-border">
                                <For
                                    each=move || recently_deleted_notes.get()
                                    key=|note| note.id
                                    let:note
                                >
                                    <div class="flex min-w-0 items-center gap-2 px-3 py-2">
                                        <span class=move || format!("min-w-0 flex-1 truncate {}", ThemeText::Subtle.classes())>
                                            {note.display_title().to_string()}
                                        </span>
                                        <button
                                            type="button"
                                            class=recovery_action_button_classes
                                            on:click=move |_| state.restore_recently_deleted_note(note.id)
                                        >
                                            "Restore"
                                        </button>
                                        <button
                                            type="button"
                                            class=recovery_danger_button_classes
                                            on:click=move |_| state.permanently_clear_recently_deleted_note(note.id)
                                            title=format!("Delete {} forever", note.display_title())
                                            aria-label=format!("Delete {} forever", note.display_title())
                                        >
                                            {recently_deleted_permanent_delete_label()}
                                        </button>
                                    </div>
                                </For>
                            </div>
                        </details>
                    </Show>
                </div>
                <SidebarBackupControls />
                <Show when=move || show_diagnostics.get()>
                    <DiagnosticsModal
                        show=show_diagnostics
                        backup_health=diagnostics_backup_health
                        corrupt_payload_status=diagnostics_corrupt_payload_status
                    />
                </Show>
            </Show>
        </div>
    }
}

#[component]
fn DiagnosticsModal(
    show: RwSignal<bool>,
    backup_health: Memo<String>,
    corrupt_payload_status: Memo<&'static str>,
) -> impl IntoView {
    let dismiss = move || show.set(false);
    let header = Box::new(move || {
        view! {
            <div>
                <h2
                    id="diagnostics-modal-title"
                    class=move || format!("text-xl font-bold {}", ThemeText::Primary.classes())
                >
                    "About Noter"
                </h2>
                <p
                    id="diagnostics-modal-description"
                    class=move || format!("mt-2 text-sm {}", ThemeText::Muted.classes())
                >
                    {app_version_label()}
                </p>
            </div>
        }
        .into_any()
    });
    let footer = Box::new(move || {
        view! {
            <button
                data-modal-cancel="true"
                type="button"
                on:click=move |_| dismiss()
                class=move || format!("min-h-10 px-5 py-2 font-semibold rounded-md transition-colors {}", ThemeState::SecondaryButton.classes())
            >
                "Close"
            </button>
        }
        .into_any()
    });
    let header_clone = header.clone();
    let footer_clone = footer.clone();

    view! {
        <Modal
            on_dismiss=dismiss
            max_width_class="max-w-md"
            header=header_clone.clone()
            footer=footer_clone.clone()
            labelledby="diagnostics-modal-title"
            describedby="diagnostics-modal-description"
            initial_focus_selector="[data-modal-cancel='true']"
        >
            <dl class="grid gap-4 p-6 text-sm sm:grid-cols-[max-content_1fr]">
                <dt class=move || format!("font-medium {}", ThemeText::Muted.classes())>"Storage"</dt>
                <dd class=move || ThemeText::Primary.classes()>{storage_mode_label()}</dd>
                <dt class=move || format!("font-medium {}", ThemeText::Muted.classes())>"Backup Health"</dt>
                <dd class=move || ThemeText::Primary.classes()>{move || backup_health.get()}</dd>
                <dt class=move || format!("font-medium {}", ThemeText::Muted.classes())>"Recovery"</dt>
                <dd class=move || ThemeText::Primary.classes()>{move || corrupt_payload_status.get()}</dd>
            </dl>
        </Modal>
    }
}

#[component]
fn NoteItem(item: NoteListItem) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let id = item.id;
    let title_highlights = item.title_highlights.clone();
    let preview_highlights = item.preview_highlights.clone();
    let tag_highlights = item.tag_highlights.clone();
    let display_date = item.display_date.clone();
    let tags = item.tags.clone();
    let tags_for_visibility = tags.clone();
    let is_pinned = item.is_pinned;
    let actions = state.note_actions(&item);
    let pin_label = actions.pin_label;
    let pin_command = actions.pin_command;
    let delete_command = actions.delete_command;
    let action_menu_open = RwSignal::new(false);

    let is_selected = move || state.selected_id() == Some(id);

    let select = move |_| {
        state.select_note_list_row(id);
    };

    let toggle_pin = move |ev: leptos::web_sys::MouseEvent| {
        ev.stop_propagation();
        action_menu_open.set(false);
        state.apply_note_list_command(pin_command);
    };

    let delete_note = move |ev: leptos::web_sys::MouseEvent| {
        ev.stop_propagation();
        action_menu_open.set(false);
        state.apply_note_list_command(delete_command);
    };

    view! {
        <div
            on:click=select
            class=move || {
                if is_selected() {
                    ui_recipes::note_row(true)
                } else {
                    ui_recipes::note_row(false)
                }
            }
        >
            <div class="flex min-w-0 justify-between items-start">
                <h3 class=note_list_title_classes>
                    <span class="block truncate">
                        {title_highlights.iter().cloned()
                            .map(|segment| {
                                if segment.is_match {
                                    view! { <mark class=move || ThemeAccent::Highlight.classes()>{segment.text}</mark> }.into_any()
                                } else {
                                    view! { <span>{segment.text}</span> }.into_any()
                                }
                            })
                            .collect_view()}
                    </span>
                </h3>
                <div class="relative flex items-center gap-1">
                    <Show when=move || is_pinned>
                        <svg xmlns="http://www.w3.org/2000/svg" class=move || format!("h-4 w-4 {}", ThemeAccent::PrimaryText.classes()) fill="currentColor" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                        </svg>
                    </Show>
                    <button
                        on:click=move |ev: leptos::web_sys::MouseEvent| {
                            ev.stop_propagation();
                            action_menu_open.update(|open| *open = !*open);
                        }
                        class=move || format!("p-1 rounded-full {}", ThemeState::NoteActionButton.classes())
                        title="Note actions"
                        aria-label="Note actions"
                        data-confirm-return="note-actions"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.75h.01M12 12h.01M12 17.25h.01" />
                        </svg>
                    </button>
                    <Show when=move || action_menu_open.get()>
                        <div
                            class=move || format!("absolute right-0 top-7 z-20 w-36 overflow-hidden rounded-md border py-1 text-sm shadow-lg {}", ThemeState::NoteActionMenu.classes())
                            on:click=move |ev| ev.stop_propagation()
                        >
                            <button
                                on:click=toggle_pin
                                class=move || format!("block w-full px-3 py-2 text-left {}", ThemeState::NoteMenuItem.classes())
                            >
                                {pin_label}
                            </button>
                            <button
                                on:click=delete_note
                                class=move || format!("block w-full px-3 py-2 text-left {}", ThemeState::DangerMenuItem.classes())
                            >
                                "Delete"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
            <div class=ui_recipes::note_list_meta_row>
                <span class=move || format!("whitespace-nowrap {}", ThemeText::Muted.classes())>{display_date}</span>
                <span class=move || format!("block truncate {}", ThemeText::Subtle.classes())>
                    {preview_highlights.iter().cloned()
                        .map(|segment| {
                            if segment.is_match {
                                view! { <mark class=move || ThemeAccent::Highlight.classes()>{segment.text}</mark> }.into_any()
                            } else {
                                view! { <span>{segment.text}</span> }.into_any()
                            }
                        })
                        .collect_view()}
                </span>
            </div>
            <Show when=move || !tags_for_visibility.is_empty()>
                <div class="mt-1.5 flex flex-wrap gap-1">
                    {tags.iter()
                        .zip(tag_highlights.iter())
                        .map(|(tag, highlights)| {
                            let tag_for_click = tag.clone();
                            let tag_segments = highlights.clone();
                            view! {
                                <button
                                    class=ui_recipes::tag_pill
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        state.select_active_tag(tag_for_click.clone());
                                    }
                                    title=tag.clone()
                                    aria-label=format!("Filter by tag {tag}")
                                >
                                    <span aria-hidden="true">"#"</span>
                                    {tag_segments.into_iter()
                                        .map(|segment| {
                                            if segment.is_match {
                                                view! { <mark class=move || ThemeAccent::Highlight.classes()>{segment.text}</mark> }.into_any()
                                            } else {
                                                view! { <span>{segment.text}</span> }.into_any()
                                            }
                                        })
                                        .collect_view()}
                                </button>
                            }
                                .into_any()
                        })
                        .collect_view()}
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        app_version_label, corrupt_payload_status, note_list_title_classes,
        recently_deleted_clear_all_button_classes, recently_deleted_permanent_delete_label,
        search_input_classes, search_syntax_hint_classes, search_syntax_hint_code_classes,
        should_show_clear_all_recently_deleted, sidebar_state_class, sidebar_title_classes,
        storage_mode_label,
    };

    #[test]
    fn compact_open_note_list_uses_the_full_viewport_width_before_desktop() {
        let class = sidebar_state_class(true);

        assert!(class.contains("w-full"));
        assert!(class.contains("lg:w-80"));
    }

    #[test]
    fn search_syntax_hint_is_a_temporary_popup_not_permanent_sidebar_content() {
        let class = search_syntax_hint_classes();
        let input = search_input_classes();
        let code = search_syntax_hint_code_classes();

        assert!(class.contains("absolute"));
        assert!(class.contains("top-full"));
        assert!(class.contains("z-30"));
        assert!(class.contains("shadow-sm"));
        assert!(class.contains("text-gray-900"));
        assert!(class.contains("dark:text-white"));
        assert!(input.contains("pr-10"));
        assert!(input.contains("bg-apple-gray-200"));
        assert!(!input.contains(&["bg", "black"].join("-")));
        assert!(code.contains("bg-apple-gray-200"));
        assert!(!code.contains(&["bg", "black"].join("-")));
    }

    #[test]
    fn clear_all_recently_deleted_is_available_only_for_non_empty_recovery_lists() {
        let button = recently_deleted_clear_all_button_classes();

        assert!(!should_show_clear_all_recently_deleted(0));
        assert!(should_show_clear_all_recently_deleted(1));
        assert!(should_show_clear_all_recently_deleted(3));
        assert!(button.contains("ml-auto"));
        assert!(button.contains("text-red"));
        assert!(button.contains("rounded-md"));
        assert_eq!(recently_deleted_permanent_delete_label(), "Delete forever");
    }

    #[test]
    fn sidebar_title_and_footer_status_remain_visible_across_themes() {
        let title = sidebar_title_classes();

        assert!(title.contains("text-gray-900"));
        assert!(title.contains("dark:text-white"));
        assert!(title.contains("text-xl"));
        assert!(title.contains("leading-[1.3]"));
    }

    #[test]
    fn diagnostics_labels_expose_release_storage_and_quarantine_state() {
        assert!(app_version_label().starts_with("Noter "));
        assert_eq!(storage_mode_label(), "Local browser storage");
        assert_eq!(corrupt_payload_status(true), "Corrupt payload quarantined");
        assert_eq!(
            corrupt_payload_status(false),
            "No corrupt payload quarantine"
        );
    }

    #[test]
    fn note_list_titles_can_truncate_inside_mobile_flex_rows() {
        let title = note_list_title_classes();

        assert!(title.contains("min-w-0"));
        assert!(title.contains("flex-1"));
        assert!(title.contains("truncate"));
        assert!(title.contains("text-sm"));
        assert!(title.contains("leading-5"));
    }
}
