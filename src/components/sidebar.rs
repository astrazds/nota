use crate::AppState;
use crate::note_discovery::NoteListItem;
use crate::note_list_interaction::{NoteListDisplayState, SEARCH_DEBOUNCE_MS};
use crate::theme::{ThemeAccent, ThemeState, ThemeSurface, ThemeText};
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{FileReader, HtmlAnchorElement, HtmlInputElement, window};

const SIDEBAR_BASE_CLASS: &str = "fixed inset-y-0 left-0 z-30 transform transition-all duration-300 ease-in-out lg:relative lg:translate-x-0 flex flex-col h-full border-r";
type TimeoutState = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let add_note = move |_| state.create_note();

    let note_projection = Memo::new(move |_| state.note_list_projection());
    let available_tags = Memo::new(move |_| state.available_tags());

    let search_input_value = RwSignal::new(state.note_search_input());
    let backup_status = RwSignal::new(String::new());
    let debounce_timeout: TimeoutState = Rc::new(RefCell::new(None));

    let export_backup = move |_| match state.export_backup_json() {
        Ok(backup_json) => match download_backup(&state.backup_file_name(), &backup_json) {
            Ok(()) => backup_status.set("Backup exported".to_string()),
            Err(message) => backup_status.set(message),
        },
        Err(_) => backup_status.set("Backup export failed".to_string()),
    };

    let import_backup = move |ev| {
        let input = event_target::<HtmlInputElement>(&ev);
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };

        backup_status.set("Importing backup...".to_string());
        let Ok(reader) = FileReader::new() else {
            backup_status.set("Backup import failed".to_string());
            input.set_value("");
            return;
        };
        let reader_for_load = reader.clone();
        let backup_status_for_load = backup_status;
        let on_load = Closure::wrap(Box::new(move |_ev: web_sys::ProgressEvent| {
            let Some(backup_json) = reader_for_load
                .result()
                .ok()
                .and_then(|value| value.as_string())
            else {
                backup_status_for_load.set("Backup import failed".to_string());
                return;
            };

            match state.import_backup_json(&backup_json) {
                Ok(()) => backup_status_for_load.set("Backup imported".to_string()),
                Err(_) => backup_status_for_load.set("Backup import failed".to_string()),
            }
        }) as Box<dyn FnMut(_)>);

        reader.set_onloadend(Some(on_load.as_ref().unchecked_ref()));
        if reader.read_as_text(&file).is_err() {
            backup_status.set("Backup import failed".to_string());
        }
        on_load.forget();
        input.set_value("");
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
                let state_class = if state.is_sidebar_open.get() {
                    "translate-x-0 w-80 max-w-full"
                } else {
                    "-translate-x-full w-0 overflow-hidden"
                };
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
                            <h1 class="text-xl font-bold">Notes</h1>
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
                                on:click=add_note
                                title="New Note"
                                class=move || format!("transition-colors p-2 {}", ThemeAccent::PrimaryText.classes())
                                aria-label="Create new note"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                </svg>
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
                            class="w-full pl-10 pr-4 py-1.5 text-sm rounded-lg focus:outline-none transition-colors bg-black/5 text-gray-900 placeholder-gray-400 focus:bg-black/10 dark:bg-white/10 dark:text-white dark:placeholder-gray-500 dark:focus:bg-white/20"
                            prop:value=move || search_input_value.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                state.edit_note_search(value.clone());
                                search_input_value.set(value);
                            }
                        />
                        <details class=move || format!("mt-1 text-xs {}", ThemeText::Subtle.classes())>
                            <summary class="inline cursor-pointer select-none hover:underline">
                                "Search syntax"
                            </summary>
                            <div class="mt-1 flex flex-wrap gap-1.5" aria-label="Supported search syntax">
                                <code class="rounded bg-black/5 px-1 py-0.5 dark:bg-white/10">"\"exact phrase\""</code>
                                <code class="rounded bg-black/5 px-1 py-0.5 dark:bg-white/10">"title:plan"</code>
                                <code class="rounded bg-black/5 px-1 py-0.5 dark:bg-white/10">"tag:work"</code>
                                <code class="rounded bg-black/5 px-1 py-0.5 dark:bg-white/10">"is:pinned"</code>
                            </div>
                        </details>
                    </div>

                    <details class=move || format!("rounded-md border border-black/10 p-2 text-xs dark:border-white/10 {}", ThemeText::Subtle.classes())>
                        <summary class="cursor-pointer select-none font-medium">"Backup"</summary>
                        <div class="mt-2 flex flex-wrap gap-2">
                            <button
                                type="button"
                                class=move || format!("rounded-md px-2 py-1 text-xs {}", ThemeState::SegmentedIdle.classes())
                                on:click=export_backup
                            >
                                "Export"
                            </button>
                            <label class=move || format!("cursor-pointer rounded-md px-2 py-1 text-xs {}", ThemeState::SegmentedIdle.classes())>
                                "Import"
                                <input
                                    type="file"
                                    accept="application/json,.json"
                                    class="sr-only"
                                    on:change=import_backup
                                />
                            </label>
                        </div>
                        <Show when=move || !backup_status.get().is_empty()>
                            <p class="mt-2">{move || backup_status.get()}</p>
                        </Show>
                    </details>

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
                            <p>No notes found</p>
                            <p class="text-sm mt-1">Try a different search term</p>
                        </div>
                    </Show>
                </div>
                <div class=move || format!("h-12 px-4 border-t border-apple-gray-300 dark:border-apple-dark-border flex items-center text-xs {}", ThemeText::Subtle.classes())>
                    <span>{move || format!("{} notes", state.note_count())}</span>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn NoteItem(item: NoteListItem) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let id = item.id;
    let title_highlights = item.title_highlights.clone();
    let preview_highlights = item.preview_highlights.clone();
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
                    format!("px-4 py-3 border-b cursor-pointer transition-all duration-200 ease-in-out group {}", ThemeState::NoteRowSelected.classes())
                } else {
                    format!("px-4 py-3 border-b cursor-pointer transition-all duration-200 ease-in-out group {}", ThemeState::NoteRowIdle.classes())
                }
            }
        >
            <div class="flex justify-between items-start">
                <h3 class=move || format!("font-semibold truncate pr-2 flex-1 {}", ThemeText::Primary.classes())>
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
            <div class="flex space-x-2 text-sm mt-1">
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
                        .map(|tag| {
                            let tag_for_click = tag.clone();
                            view! {
                                <button
                                    class=move || format!("px-1.5 py-0.5 text-xs rounded-full {}", ThemeState::TagPill.classes())
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        state.select_active_tag(tag_for_click.clone());
                                    }
                                    title=tag.clone()
                                    aria-label=format!("Filter by tag {tag}")
                                >
                                    {format!("#{tag}")}
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

fn download_backup(filename: &str, backup_json: &str) -> Result<(), String> {
    let document = window()
        .and_then(|window| window.document())
        .ok_or_else(|| "Backup export failed".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Backup export failed".to_string())?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "Backup export failed".to_string())?;

    anchor.set_download(filename);
    anchor.set_href(&format!(
        "data:application/json;charset=utf-8,{}",
        percent_encode_data_url(backup_json)
    ));
    anchor.click();
    Ok(())
}

fn percent_encode_data_url(value: &str) -> String {
    value
        .bytes()
        .fold(String::with_capacity(value.len()), |mut encoded, byte| {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char)
                }
                _ => encoded.push_str(&format!("%{byte:02X}")),
            }
            encoded
        })
}

#[cfg(test)]
mod tests {
    use super::percent_encode_data_url;

    #[test]
    fn backup_download_data_url_percent_encodes_json_content() {
        assert_eq!(
            percent_encode_data_url("{\"title\":\"日本語 note\"}"),
            "%7B%22title%22%3A%22%E6%97%A5%E6%9C%AC%E8%AA%9E%20note%22%7D"
        );
    }
}
