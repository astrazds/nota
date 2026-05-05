use crate::model::{collect_note_tags, filter_and_sort_notes};
use crate::{AppState, SaveStatus};
use chrono::Utc;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::window;

const MOBILE_BREAKPOINT: f64 = 1024.0;
const SEARCH_DEBOUNCE_MS: i32 = 200;
const SIDEBAR_BASE_CLASS: &str = "fixed inset-y-0 left-0 z-30 transform transition-all duration-300 ease-in-out lg:relative lg:translate-x-0 flex flex-col h-full border-r bg-apple-gray-100 border-apple-gray-300 dark:bg-apple-dark-sidebar dark:border-apple-dark-border";
type TimeoutState = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

fn highlight_segments(text: &str, query: &str) -> Vec<(String, bool)> {
    let match_ranges = find_case_insensitive_match_ranges(text, query);
    if match_ranges.is_empty() {
        return vec![(text.to_string(), false)];
    }

    let mut segments = Vec::with_capacity(match_ranges.len() * 2 + 1);
    let mut last_end = 0;

    for (start, end) in match_ranges {
        if start > last_end {
            segments.push((text[last_end..start].to_string(), false));
        }
        segments.push((text[start..end].to_string(), true));
        last_end = end;
    }

    if last_end < text.len() {
        segments.push((text[last_end..].to_string(), false));
    }

    segments
}

fn find_case_insensitive_match_ranges(text: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }

    if text.is_ascii() && query.is_ascii() {
        let text_lower = text.to_ascii_lowercase();
        let query_lower = query.to_ascii_lowercase();
        let mut ranges = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = text_lower[search_start..].find(&query_lower) {
            let start = search_start + pos;
            let end = start + query_lower.len();
            ranges.push((start, end));
            search_start = end;

            if search_start >= text_lower.len() {
                break;
            }
        }

        return ranges;
    }

    find_case_insensitive_match_ranges_unicode(text, query)
}

fn find_case_insensitive_match_ranges_unicode(text: &str, query: &str) -> Vec<(usize, usize)> {
    let query_lower = query.to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }

    let mut boundaries: Vec<usize> = text.char_indices().map(|(idx, _)| idx).collect();
    boundaries.push(text.len());

    let query_lower_len = query_lower.len();
    let mut ranges = Vec::new();
    let mut i = 0;

    while i + 1 < boundaries.len() {
        let start = boundaries[i];
        let mut matched_end = None;

        for &end in &boundaries[i + 1..] {
            let candidate_lower = text[start..end].to_lowercase();
            if candidate_lower.starts_with(&query_lower) {
                matched_end = Some(end);
                break;
            }

            if candidate_lower.len() >= query_lower_len {
                break;
            }
        }

        if let Some(end) = matched_end {
            ranges.push((start, end));
            if let Ok(next_index) = boundaries.binary_search(&end) {
                i = next_index;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    ranges
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");

    let add_note = move |_| {
        let new_note = crate::model::Note::new("".to_string(), "".to_string());
        let id = new_note.id;
        state.notes.update(|n| n.insert(0, new_note));
        state.selected_id.set(Some(id));
        state.focus_title_request.set(true);
    };

    let filtered_notes = Memo::new(move |_| {
        let query = state.search_query.get();
        let active_tag = state.active_tag.get();
        let notes = state.notes.get();
        filter_and_sort_notes(&notes, &query, active_tag.as_deref())
    });
    let available_tags = Memo::new(move |_| collect_note_tags(&state.notes.get()));

    let search_input_value = RwSignal::new(state.search_query.get());
    let debounce_timeout: TimeoutState = Rc::new(RefCell::new(None));

    Effect::new(move |_| {
        let input = search_input_value.get();
        let query_for_closure = input.clone();
        let timeout_ref = debounce_timeout.clone();

        if let Some((id, _)) = timeout_ref.borrow_mut().take() {
            if let Some(win) = window() {
                win.clear_timeout_with_handle(id);
            }
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
                    "translate-x-0 w-80"
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
                                on:click=move |_| state.is_dark_mode.update(|v| *v = !*v)
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
                                on:click=move |_| state.is_sidebar_open.update(|v| *v = !*v)
                                class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
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

                    <Show when=move || !available_tags.get().is_empty()>
                        <div class="flex flex-wrap gap-1.5">
                            <button
                                class=move || {
                                    if state.active_tag.get().is_none() {
                                        "px-2 py-0.5 text-xs rounded-full bg-apple-yellow text-white"
                                    } else {
                                        "px-2 py-0.5 text-xs rounded-full bg-black/5 text-gray-600 hover:bg-black/10 dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20"
                                    }
                                }
                                on:click=move |_| state.active_tag.set(None)
                                title="Show all tags"
                                aria-label="Show all tags"
                            >
                                "All"
                            </button>
                            <For
                                each=move || available_tags.get()
                                key=|tag| tag.clone()
                                let:tag
                            >
                                <button
                                    class={
                                        let tag_for_class = tag.clone();
                                        move || {
                                            if state.active_tag.get().as_deref() == Some(tag_for_class.as_str()) {
                                                "px-2 py-0.5 text-xs rounded-full bg-apple-yellow text-white"
                                            } else {
                                                "px-2 py-0.5 text-xs rounded-full bg-black/5 text-gray-600 hover:bg-black/10 dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20"
                                            }
                                        }
                                    }
                                    on:click={
                                        let tag_for_click = tag.clone();
                                        move |_| state.active_tag.set(Some(tag_for_click.clone()))
                                    }
                                    title={
                                        let tag_for_title = tag.clone();
                                        move || format!("Filter by {tag_for_title}")
                                    }
                                    aria-label={
                                        let tag_for_aria = tag.clone();
                                        move || format!("Filter notes by tag {tag_for_aria}")
                                    }
                                >
                                    {format!("#{tag}")}
                                </button>
                            </For>
                        </div>
                    </Show>
                </div>
                <div class="flex-1 overflow-y-auto pb-4">
                    <Show when=move || !filtered_notes.get().is_empty()>
                        <For
                            each=move || filtered_notes.get()
                            key=|note_id| *note_id
                            let:note_id
                        >
                            <NoteItem id=note_id />
                        </For>
                    </Show>
                    <Show when=move || {
                        filtered_notes.get().is_empty()
                            && (!state.search_query.get().is_empty() || state.active_tag.get().is_some())
                    }>
                        <div class="p-8 text-center text-gray-400 dark:text-gray-500">
                            <p>No notes found</p>
                            <p class="text-sm mt-1">Try a different search term</p>
                        </div>
                    </Show>
                </div>
                <div class="h-16 px-4 border-t border-apple-gray-300 text-gray-400 dark:border-apple-dark-border dark:text-gray-500 flex items-center">
                    <div class="w-full grid grid-cols-3 items-center text-xs">
                        <span class="justify-self-start">{move || format!("{} notes", state.notes.get().len())}</span>
                        <span
                            class="justify-self-center text-center"
                            class:text-apple-yellow=move || matches!(state.save_status.get(), SaveStatus::Saving)
                            class:text-green-500=move || matches!(state.save_status.get(), SaveStatus::Saved)
                        >
                            {move || match state.save_status.get() {
                                SaveStatus::Saving => "Saving...",
                                SaveStatus::Saved => "Saved",
                            }}
                        </span>
                        <span class="justify-self-end">{env!("CARGO_PKG_VERSION")}</span>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn NoteItem(id: Uuid) -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let note = Memo::new(move |_| state.notes.get().iter().find(|n| n.id == id).cloned());
    let is_selected = move || state.selected_id.get() == Some(id);

    let select = move |_| {
        state.selected_id.set(Some(id));
        if let Some(win) = window() {
            if win
                .inner_width()
                .unwrap_or_default()
                .as_f64()
                .unwrap_or(MOBILE_BREAKPOINT)
                < MOBILE_BREAKPOINT
            {
                state.is_sidebar_open.set(false);
            }
        }
    };

    let toggle_pin = move |ev: leptos::web_sys::MouseEvent| {
        ev.stop_propagation();
        state.notes.update(|notes| {
            if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
                note.is_pinned = !note.is_pinned;
                note.last_modified = Utc::now();
            }
        });
    };

    let delete_note = move |ev: leptos::web_sys::MouseEvent| {
        ev.stop_propagation();
        state.selected_id.set(Some(id));
        state.show_delete_confirm.set(true);
    };

    view! {
        <div
            on:click=select
            class=move || {
                if is_selected() {
                    "p-4 border-b border-apple-gray-200 dark:border-apple-dark-border cursor-pointer transition-all duration-200 ease-in-out group bg-apple-yellow/20 dark:bg-apple-yellow/30"
                } else {
                    "p-4 border-b border-apple-gray-200 dark:border-apple-dark-border cursor-pointer transition-all duration-200 ease-in-out group hover:bg-apple-gray-200 dark:hover:bg-white/5"
                }
            }
        >
            <div class="flex justify-between items-start">
                <h3 class="font-semibold truncate pr-2 flex-1 text-gray-900 dark:text-white">
                    <span class="block truncate">
                        {move || {
                            let title = note
                                .get()
                                .map(|n| n.display_title().to_string())
                                .unwrap_or_default();
                            let query = state.search_query.get();
                            highlight_segments(&title, &query)
                                .into_iter()
                                .map(|(segment, is_match)| {
                                    if is_match {
                                        view! { <mark class="bg-apple-yellow/30">{segment}</mark> }.into_any()
                                    } else {
                                        view! { <span>{segment}</span> }.into_any()
                                    }
                                })
                                .collect_view()
                        }}
                    </span>
                </h3>
                <div class="flex items-center space-x-1">
                    <button
                        on:click=toggle_pin
                        class=move || {
                            let is_pinned = note.get().map(|n| n.is_pinned).unwrap_or(false);
                            if is_pinned {
                                "opacity-100 transition-opacity p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 text-apple-yellow"
                            } else {
                                "opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 text-gray-400"
                            }
                        }
                        title=move || if note.get().map(|n| n.is_pinned).unwrap_or(false) { "Unpin Note" } else { "Pin Note" }
                        aria-label=move || if note.get().map(|n| n.is_pinned).unwrap_or(false) { "Unpin note" } else { "Pin note" }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill=move || if note.get().map(|n| n.is_pinned).unwrap_or(false) { "currentColor" } else { "none" } viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                        </svg>
                    </button>
                    <button
                        on:click=delete_note
                        class="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 text-gray-400 hover:text-red-500"
                        title="Delete Note"
                        aria-label="Delete note"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>
            </div>
            <div class="flex space-x-2 text-sm mt-1">
                <span class="whitespace-nowrap text-gray-500 dark:text-gray-400">
                    {move || note.get().map(|n| n.display_date()).unwrap_or_default()}
                </span>
                <span class="block truncate text-gray-400 dark:text-gray-500">
                    {move || {
                        let preview = note.get().map(|n| n.preview()).unwrap_or_default();
                        let query = state.search_query.get();
                        highlight_segments(&preview, &query)
                            .into_iter()
                            .map(|(segment, is_match)| {
                                if is_match {
                                    view! { <mark class="bg-apple-yellow/30">{segment}</mark> }
                                        .into_any()
                                } else {
                                    view! { <span>{segment}</span> }.into_any()
                                }
                            })
                            .collect_view()
                    }}
                </span>
            </div>
            <Show when=move || note.get().is_some_and(|n| !n.tags.is_empty())>
                <div class="mt-2 flex flex-wrap gap-1">
                    {move || {
                        let tags = note.get().map(|n| n.tags.clone()).unwrap_or_default();
                        tags.into_iter()
                            .map(|tag| {
                                let tag_for_click = tag.clone();
                                view! {
                                    <button
                                        class="px-2 py-0.5 text-xs rounded-full bg-black/5 text-gray-500 hover:bg-black/10 dark:bg-white/10 dark:text-gray-400 dark:hover:bg-white/20"
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
                            .collect_view()
                    }}
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::highlight_segments;

    #[test]
    fn should_split_ascii_case_insensitive_matches() {
        let highlighted = highlight_segments("Hello World", "world");
        assert_eq!(
            highlighted,
            vec![("Hello ".to_string(), false), ("World".to_string(), true),]
        );
    }

    #[test]
    fn should_not_panic_on_unicode_case_mapping() {
        let highlighted = std::panic::catch_unwind(|| highlight_segments("İstanbul", "i"));
        assert!(highlighted.is_ok());
    }

    #[test]
    fn should_highlight_unicode_case_folded_matches() {
        let highlighted = highlight_segments("İstanbul", "i");
        assert_eq!(
            highlighted,
            vec![("İ".to_string(), true), ("stanbul".to_string(), false)]
        );
    }
}
