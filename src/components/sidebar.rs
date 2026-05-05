use crate::AppState;
use crate::note_discovery::{NoteListItem, collect_note_tags, project_note_list};
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::window;

const MOBILE_BREAKPOINT: f64 = 1024.0;
const SEARCH_DEBOUNCE_MS: i32 = 200;
const SIDEBAR_BASE_CLASS: &str = "fixed inset-y-0 left-0 z-30 transform transition-all duration-300 ease-in-out lg:relative lg:translate-x-0 flex flex-col h-full border-r bg-apple-gray-100 border-apple-gray-300 dark:bg-apple-dark-sidebar dark:border-apple-dark-border";
type TimeoutState = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let add_note = move |_| state.create_note();

    let note_projection = Memo::new(move |_| {
        let query = state.search_query.get();
        let active_tag = state.active_tag.get();
        let notes = state.notes.get();
        project_note_list(
            &notes,
            state.selected_id.get(),
            &query,
            active_tag.as_deref(),
        )
    });
    let available_tags = Memo::new(move |_| collect_note_tags(&state.notes.get()));

    let search_input_value = RwSignal::new(state.search_query.get());
    let debounce_timeout: TimeoutState = Rc::new(RefCell::new(None));

    Effect::new(move |_| {
        let input = search_input_value.get();
        let query_for_closure = input.clone();
        let timeout_ref = debounce_timeout.clone();

        if let Some((id, _)) = timeout_ref.borrow_mut().take()
            && let Some(win) = window()
        {
            win.clear_timeout_with_handle(id);
        }

        if let Some(win) = window() {
            let closure = Closure::wrap(Box::new(move || {
                state.search_query.set(query_for_closure.clone());
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
                format!("{SIDEBAR_BASE_CLASS} {state_class}")
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
                                class="p-2 rounded-full hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
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
                                class="p-2 lg:hidden text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
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
                                class="text-apple-yellow hover:text-yellow-600 transition-colors p-2"
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
                            on:input=move |ev| search_input_value.set(event_target_value(&ev))
                        />
                    </div>

                    <Show when=move || state.active_tag.get().is_some() && !available_tags.get().is_empty()>
                        <div class="flex items-center gap-2 text-xs">
                            <span class="text-gray-500 dark:text-gray-400">"Filtered by"</span>
                            <button
                                class="px-2 py-0.5 rounded-full bg-apple-yellow text-white"
                                on:click=move |_| state.active_tag.set(None)
                                title="Clear tag filter"
                                aria-label="Clear tag filter"
                            >
                                {move || state.active_tag.get().map(|tag| format!("#{tag}")).unwrap_or_else(|| "All".to_string())}
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
                        projection.rows.is_empty() && projection.has_active_filter
                    }>
                        <div class="p-8 text-center text-gray-400 dark:text-gray-500">
                            <p>No notes found</p>
                            <p class="text-sm mt-1">Try a different search term</p>
                        </div>
                    </Show>
                </div>
                <div class="h-12 px-4 border-t border-apple-gray-300 text-gray-400 dark:border-apple-dark-border dark:text-gray-500 flex items-center text-xs">
                    <span>{move || format!("{} notes", state.notes.get().len())}</span>
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
    let action_menu_open = RwSignal::new(false);

    let is_selected = move || state.selected_id.get() == Some(id);

    let select = move |_| {
        state.select_note(id);
        if let Some(win) = window()
            && win
                .inner_width()
                .unwrap_or_default()
                .as_f64()
                .unwrap_or(MOBILE_BREAKPOINT)
                < MOBILE_BREAKPOINT
        {
            state.is_sidebar_open.set(false);
        }
    };

    let toggle_pin = move |ev: leptos::web_sys::MouseEvent| {
        ev.stop_propagation();
        action_menu_open.set(false);
        state.toggle_note_pin(id);
    };

    let delete_note = move |ev: leptos::web_sys::MouseEvent| {
        ev.stop_propagation();
        action_menu_open.set(false);
        state.request_delete_note(id);
    };

    view! {
        <div
            on:click=select
            class=move || {
                if is_selected() {
                    "px-4 py-3 border-b border-apple-gray-200 dark:border-apple-dark-border cursor-pointer transition-all duration-200 ease-in-out group bg-apple-yellow/15 dark:bg-apple-yellow/20"
                } else {
                    "px-4 py-3 border-b border-apple-gray-200 dark:border-apple-dark-border cursor-pointer transition-all duration-200 ease-in-out group hover:bg-apple-gray-200 dark:hover:bg-white/5"
                }
            }
        >
            <div class="flex justify-between items-start">
                <h3 class="font-semibold truncate pr-2 flex-1 text-gray-900 dark:text-white">
                    <span class="block truncate">
                        {title_highlights.iter().cloned()
                            .map(|segment| {
                                if segment.is_match {
                                    view! { <mark class="bg-apple-yellow/30">{segment.text}</mark> }.into_any()
                                } else {
                                    view! { <span>{segment.text}</span> }.into_any()
                                }
                            })
                            .collect_view()}
                    </span>
                </h3>
                <div class="relative flex items-center gap-1">
                    <Show when=move || is_pinned>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-apple-yellow" fill="currentColor" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                        </svg>
                    </Show>
                    <button
                        on:click=move |ev: leptos::web_sys::MouseEvent| {
                            ev.stop_propagation();
                            action_menu_open.update(|open| *open = !*open);
                        }
                        class="p-1 rounded-full text-gray-400 hover:text-gray-700 hover:bg-black/10 dark:hover:text-gray-200 dark:hover:bg-white/10"
                        title="Note actions"
                        aria-label="Note actions"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.75h.01M12 12h.01M12 17.25h.01" />
                        </svg>
                    </button>
                    <Show when=move || action_menu_open.get()>
                        <div
                            class="absolute right-0 top-7 z-20 w-36 overflow-hidden rounded-md border border-apple-gray-200 bg-white py-1 text-sm shadow-lg dark:border-apple-dark-border dark:bg-apple-dark-sidebar"
                            on:click=move |ev| ev.stop_propagation()
                        >
                            <button
                                on:click=toggle_pin
                                class="block w-full px-3 py-2 text-left text-gray-700 hover:bg-black/5 dark:text-gray-200 dark:hover:bg-white/10"
                            >
                                {if is_pinned { "Unpin" } else { "Pin" }}
                            </button>
                            <button
                                on:click=delete_note
                                class="block w-full px-3 py-2 text-left text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-500/10"
                            >
                                "Delete"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
            <div class="flex space-x-2 text-sm mt-1">
                <span class="whitespace-nowrap text-gray-500 dark:text-gray-400">{display_date}</span>
                <span class="block truncate text-gray-400 dark:text-gray-500">
                    {preview_highlights.iter().cloned()
                        .map(|segment| {
                            if segment.is_match {
                                view! { <mark class="bg-apple-yellow/30">{segment.text}</mark> }.into_any()
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
                                    class="px-1.5 py-0.5 text-xs rounded-full bg-black/5 text-gray-500 hover:bg-black/10 dark:bg-white/10 dark:text-gray-400 dark:hover:bg-white/20"
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        state.active_tag.set(Some(tag_for_click.clone()));
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
