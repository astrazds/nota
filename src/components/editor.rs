use crate::AppState;
use crate::components::CheatsheetModal;
use crate::editor_view::EditorViewMode;
use crate::markdown_editing::{BrowserSelection, MarkdownCommand, apply_markdown_command};
use crate::markdown_preview::render_markdown_preview;
use crate::model::Note;
use crate::note_workspace::{FocusIntent, WorkspaceDisplayState};
use crate::storage::SaveStatus;
use crate::tag_rules::{parse_tags_input, tags_to_input};
use crate::theme::{ThemeAccent, ThemeState, ThemeSurface, ThemeText};
use leptos::prelude::*;

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let show_cheatsheet = RwSignal::new(false);

    let title_input_ref = NodeRef::<leptos::html::Input>::new();
    let tags_input_ref = NodeRef::<leptos::html::Input>::new();
    let content_area_ref = NodeRef::<leptos::html::Textarea>::new();
    let tags_input_value = RwSignal::new(String::new());
    let is_editing_tags = RwSignal::new(false);

    let selected_note = Memo::new(move |_| state.selected_note());
    let workspace_display_state = Memo::new(move |_| state.workspace_display_state());

    Effect::new(move |_| {
        if state.focus_intent() == FocusIntent::NoteTitle {
            if let Some(input) = title_input_ref.get() {
                let _ = input.focus();
            }
            state.take_focus_intent();
        }
    });

    Effect::new(move |_| {
        if is_editing_tags.get() {
            return;
        }

        let tags_value = state
            .selected_note()
            .map(|note| tags_to_input(&note.tags))
            .unwrap_or_default();
        tags_input_value.set(tags_value);
    });

    let on_input_content = move |ev| {
        let value = event_target_value(&ev);
        state.update_selected_content(value);
    };

    let on_input_title = move |ev| {
        let value = event_target_value(&ev);
        state.update_selected_title(value);
    };

    let on_input_tags = move |ev| {
        let value = event_target_value(&ev);
        tags_input_value.set(value.clone());

        let parsed_tags = parse_tags_input(&value);
        state.update_selected_tags(parsed_tags);
    };

    let tag_suggestions = Memo::new(move |_| {
        if is_editing_tags.get() {
            state.tag_suggestions(&tags_input_value.get())
        } else {
            Vec::new()
        }
    });
    let tag_cleanup_plan = Memo::new(move |_| state.tag_cleanup_plan());

    let apply_tags_value = move |value: String| {
        let parsed_tags = parse_tags_input(&value);
        tags_input_value.set(value);
        state.update_selected_tags(parsed_tags);
        if let Some(input) = tags_input_ref.get() {
            let _ = input.focus();
        }
    };

    let on_tags_keydown = move |ev: leptos::web_sys::KeyboardEvent| {
        let key = ev.key();
        if key != "Enter" && key != "Tab" {
            return;
        }

        let Some(suggestion) = tag_suggestions.get().into_iter().next() else {
            return;
        };

        ev.prevent_default();
        apply_tags_value(suggestion.completed_input);
    };

    let commit_tags_input = move || {
        let parsed_tags = parse_tags_input(&tags_input_value.get_untracked());
        if state.selected_note().is_some() {
            let normalised_input = tags_to_input(&parsed_tags);
            state.update_selected_tags(parsed_tags);
            tags_input_value.set(normalised_input);
        }
    };

    let markdown_html = Memo::new(move |_| {
        let note = selected_note.get();
        let title = note
            .as_ref()
            .map(|n: &Note| n.display_title().to_string())
            .unwrap_or_default();
        let content = note
            .as_ref()
            .map(|n: &Note| n.content.as_str())
            .unwrap_or_default();

        render_markdown_preview(&title, content)
    });

    let apply_format = move |command: MarkdownCommand| {
        if let Some(textarea) = content_area_ref.get() {
            let start_utf16 = textarea.selection_start().unwrap_or_default().unwrap_or(0);
            let end_utf16 = textarea.selection_end().unwrap_or_default().unwrap_or(0);
            let content = textarea.value();
            let formatted = apply_markdown_command(
                &content,
                BrowserSelection {
                    start_utf16: start_utf16 as usize,
                    end_utf16: end_utf16 as usize,
                },
                command,
            );

            if selected_note.get_untracked().is_some() {
                state.update_selected_content(formatted.content.clone());
            }

            let _ = textarea.focus();
            let new_cursor_pos = formatted.caret_utf16 as u32;
            let _ = textarea.set_selection_start(Some(new_cursor_pos));
            let _ = textarea.set_selection_end(Some(new_cursor_pos));
        }
    };

    view! {
        <div class="flex-1 flex flex-col h-full overflow-hidden relative transition-colors duration-300">
            <CheatsheetModal show=show_cheatsheet />

            <div class=move || format!("min-h-14 p-2 px-3 md:px-4 flex justify-between items-center gap-3 border-b sticky top-0 z-10 {}", ThemeSurface::EditorChrome.classes())>
                <div class="flex items-center gap-2 min-w-0">
                    <button
                        on:click=move |_| state.toggle_sidebar()
                        class=move || format!("p-2 lg:hidden {}", ThemeAccent::PrimaryText.classes())
                        title="Toggle Sidebar"
                        aria-label="Toggle sidebar"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                        </svg>
                    </button>
                    <div class="flex space-x-1 border-r pr-3 border-gray-200 dark:border-apple-dark-border">
                        <button
                            on:click=move |_| state.set_editor_view_mode(EditorViewMode::Write)
                            title="Write"
                            aria-label="Write mode"
                            aria-pressed=move || state.editor_view_mode.get() == EditorViewMode::Write
                            class=move || {
                                if state.editor_view_mode.get() == EditorViewMode::Write {
                                    format!("px-2.5 py-1 text-sm rounded-md transition-all {}", ThemeState::SegmentedActive.classes())
                                } else {
                                    format!("px-2.5 py-1 text-sm rounded-md transition-all {}", ThemeState::SegmentedIdle.classes())
                                }
                            }
                        >
                            "Write"
                        </button>
                        <button
                            on:click=move |_| state.set_editor_view_mode(EditorViewMode::Preview)
                            title="Preview"
                            aria-label="Preview mode"
                            aria-pressed=move || state.editor_view_mode.get() == EditorViewMode::Preview
                            class=move || {
                                if state.editor_view_mode.get() == EditorViewMode::Preview {
                                    format!("px-2.5 py-1 text-sm rounded-md transition-all {}", ThemeState::SegmentedActive.classes())
                                } else {
                                    format!("px-2.5 py-1 text-sm rounded-md transition-all {}", ThemeState::SegmentedIdle.classes())
                                }
                            }
                        >
                            "Preview"
                        </button>
                        <button
                            on:click=move |_| state.set_editor_view_mode(EditorViewMode::Split)
                            title="Split"
                            aria-label="Split mode"
                            aria-pressed=move || state.editor_view_mode.get() == EditorViewMode::Split
                            class=move || {
                                if state.editor_view_mode.get() == EditorViewMode::Split {
                                    format!("hidden lg:inline-flex px-2.5 py-1 text-sm rounded-md transition-all {}", ThemeState::SegmentedActive.classes())
                                } else {
                                    format!("hidden lg:inline-flex px-2.5 py-1 text-sm rounded-md transition-all {}", ThemeState::SegmentedIdle.classes())
                                }
                            }
                        >
                            "Split"
                        </button>
                        <button
                            on:click=move |_| show_cheatsheet.set(true)
                            title="Markdown Help"
                            aria-label="Show markdown cheatsheet"
                            class=move || format!("px-3 py-1 text-sm rounded-md transition-colors {}", ThemeState::SegmentedIdle.classes())
                        >
                            "?"
                        </button>
                    </div>

                    <Show when=move || state.editor_view_mode.get() != EditorViewMode::Preview>
                        <div class="hidden sm:flex items-center space-x-1">
                            <ToolbarButton on_click=move |_| apply_format(MarkdownCommand::Bold) title="Bold" aria_label="Bold">
                                <span class="font-bold">B</span>
                            </ToolbarButton>
                            <ToolbarButton on_click=move |_| apply_format(MarkdownCommand::Italic) title="Italic" aria_label="Italic">
                                <span class="italic">I</span>
                            </ToolbarButton>
                            <ToolbarButton on_click=move |_| apply_format(MarkdownCommand::Strikethrough) title="Strikethrough" aria_label="Strikethrough">
                                <span class="line-through">S</span>
                            </ToolbarButton>
                            <ToolbarButton on_click=move |_| apply_format(MarkdownCommand::TaskList) title="Task List" aria_label="Task list">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h4v4H4V6zm0 8h4v4H4v-4zm0 8h4v-4H4v4zM12 7h8M12 15h8M12 19h8" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 7.5l1.2 1.2L7.8 7" />
                                </svg>
                            </ToolbarButton>
                            <ToolbarButton
                                on_click=move |_| apply_format(MarkdownCommand::Table)
                                title="Insert Table"
                                aria_label="Insert table"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6h18v12H3V6zM3 12h18M9 6v12M15 6v12" />
                                </svg>
                            </ToolbarButton>
                        </div>
                    </Show>
                </div>
                <span
                    class="hidden sm:inline shrink-0 text-xs text-gray-500 dark:text-gray-400"
                    class:text-apple-yellow=move || matches!(state.save_status.get(), SaveStatus::Saving)
                    class:text-green-600=move || matches!(state.save_status.get(), SaveStatus::Saved)
                >
                    {move || match state.save_status.get() {
                        SaveStatus::Saving => "Saving...",
                        SaveStatus::Saved => "Saved",
                    }}
                </span>
            </div>

            <div class="flex-1 flex overflow-hidden">
                {move || match workspace_display_state.get() {
                    WorkspaceDisplayState::NoteSelected => view! {
                        <div class="flex-1 flex overflow-hidden divide-x divide-apple-gray-200 dark:divide-apple-dark-border">
                            <Show when=move || state.editor_view_mode.get().surfaces().writing>
                                <div class=move || format!("flex-1 flex flex-col overflow-hidden {}", ThemeSurface::WritingSurface.classes())>
                                    <div class="px-6 pt-7 pb-3 md:px-8 md:pt-8 space-y-3 border-b border-transparent">
                                        <input
                                            node_ref=title_input_ref
                                            type="text"
                                            class=move || format!("w-full min-w-0 text-2xl md:text-3xl font-bold leading-tight focus:outline-none bg-transparent {} {}", ThemeText::Primary.classes(), ThemeText::Placeholder.classes())
                                            placeholder="Note Title"
                                            prop:value=move || selected_note.get().map(|note| note.title).unwrap_or_default()
                                            on:input=on_input_title
                                        />
                                        <input
                                            node_ref=tags_input_ref
                                            type="text"
                                            class=move || format!("w-full max-w-xl px-0 py-1 text-sm focus:outline-none bg-transparent placeholder-gray-400 dark:placeholder-gray-600 {}", ThemeText::Muted.classes())
                                            placeholder="Tags"
                                            prop:value=move || tags_input_value.get()
                                            on:focus=move |_| is_editing_tags.set(true)
                                            on:input=on_input_tags
                                            on:keydown=on_tags_keydown
                                            on:blur=move |_| {
                                                is_editing_tags.set(false);
                                                commit_tags_input();
                                            }
                                        />
                                        <Show when=move || selected_note.get().is_some_and(|note| !note.tags.is_empty())>
                                            <div class="flex flex-wrap gap-1.5">
                                                {move || {
                                                    selected_note
                                                        .get()
                                                        .map(|note| {
                                                            note.tags
                                                                .into_iter()
                                                                .map(|tag| {
                                                                    let tag_for_remove = tag.clone();
                                                                    view! {
                                                                        <span class=move || format!("inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs {}", ThemeState::TagPill.classes())>
                                                                            {format!("#{tag}")}
                                                                            <button
                                                                                type="button"
                                                                                class="ml-0.5 rounded-full px-1 leading-none opacity-70 hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-apple-blue"
                                                                                title=format!("Remove tag {tag_for_remove}")
                                                                                aria-label=format!("Remove tag {tag_for_remove}")
                                                                                on:click=move |_| {
                                                                                    state.remove_selected_tag(&tag_for_remove);
                                                                                }
                                                                            >
                                                                                "x"
                                                                            </button>
                                                                        </span>
                                                                    }
                                                                })
                                                                .collect_view()
                                                        })
                                                }}
                                            </div>
                                        </Show>
                                        <Show when=move || !tag_suggestions.get().is_empty()>
                                            <div class=move || format!("max-w-xl overflow-hidden rounded-md border shadow-sm {}", ThemeSurface::EditorChrome.classes())>
                                                {move || {
                                                    tag_suggestions
                                                        .get()
                                                        .into_iter()
                                                        .map(|suggestion| {
                                                            let completed_input = suggestion.completed_input.clone();
                                                            view! {
                                                                <button
                                                                    type="button"
                                                                    class=move || format!("block w-full px-3 py-2 text-left text-sm transition-colors {}", ThemeState::SegmentedIdle.classes())
                                                                    on:mousedown=move |ev| ev.prevent_default()
                                                                    on:click=move |_| apply_tags_value(completed_input.clone())
                                                                >
                                                                    {format!("#{label}", label = suggestion.label)}
                                                                </button>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </div>
                                        </Show>
                                        <Show when=move || !tag_cleanup_plan.get().is_empty()>
                                            <details class=move || format!("max-w-xl rounded-md border p-3 text-sm {}", ThemeSurface::EditorChrome.classes())>
                                                <summary class="cursor-pointer select-none">
                                                    "Review Tag cleanup"
                                                </summary>
                                                <div class="mt-2 space-y-2">
                                                    {move || {
                                                        tag_cleanup_plan
                                                            .get()
                                                            .changes
                                                            .into_iter()
                                                            .take(4)
                                                            .map(|change| {
                                                                view! {
                                                                    <p class=move || ThemeText::Muted.classes()>
                                                                        {format!(
                                                                            "{} -> {}",
                                                                            tags_to_input(&change.before),
                                                                            tags_to_input(&change.after),
                                                                        )}
                                                                    </p>
                                                                }
                                                            })
                                                            .collect_view()
                                                    }}
                                                    <button
                                                        type="button"
                                                        class=move || format!("rounded-md px-3 py-1 text-sm {}", ThemeState::SegmentedIdle.classes())
                                                        on:click=move |_| {
                                                            let plan = tag_cleanup_plan.get_untracked();
                                                            state.apply_tag_cleanup(&plan);
                                                        }
                                                    >
                                                        "Apply cleanup"
                                                    </button>
                                                </div>
                                            </details>
                                        </Show>
                                    </div>
                                    <textarea
                                        node_ref=content_area_ref
                                        class=move || format!("flex-1 px-6 pb-8 pt-3 md:px-8 text-base md:text-lg leading-8 focus:outline-none resize-none bg-transparent font-mono dark:text-gray-300 {}", ThemeAccent::Selection.classes())
                                        placeholder="Start typing..."
                                        prop:value=move || selected_note.get().map(|note| note.content).unwrap_or_default()
                                        on:input=on_input_content
                                    ></textarea>
                                </div>
                            </Show>

                            <Show when=move || state.editor_view_mode.get().surfaces().preview>
                                <div
                                    class=move || {
                                        if state.editor_view_mode.get() == EditorViewMode::Split {
                                            format!("flex-1 px-6 pb-8 pt-7 md:px-8 md:pt-8 overflow-y-auto prose max-w-none break-words shadow-inner border-l transition-colors {}", ThemeSurface::SplitPreview.classes())
                                        } else {
                                            format!("flex-1 px-6 pb-8 pt-7 md:px-8 md:pt-8 overflow-y-auto prose max-w-none break-words transition-colors {}", ThemeSurface::Preview.classes())
                                        }
                                    }
                                >
                                    <div inner_html=markdown_html.get()></div>
                                </div>
                            </Show>
                        </div>
                    }.into_any(),
                    WorkspaceDisplayState::EmptyCollection => view! {
                        <div class=move || format!("flex-1 flex items-center justify-center px-6 {}", ThemeState::EmptyState.classes())>
                            <div class="text-center max-w-sm">
                                <svg xmlns="http://www.w3.org/2000/svg" class=move || format!("h-16 w-16 mx-auto mb-5 {}", ThemeState::EmptyIllustration.classes()) fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 4v16m8-8H4" />
                                </svg>
                                <h2 class=move || format!("text-2xl font-semibold {}", ThemeText::Primary.classes())>"Create your first note"</h2>
                                <p class="mt-2 text-sm leading-6">"Start with a title, then write in Markdown when you need it."</p>
                                <button
                                    on:click=move |_| state.create_note()
                                    class=move || format!("mt-6 inline-flex items-center rounded-md px-4 py-2 text-sm font-semibold transition-colors {} {}", ThemeAccent::PrimaryFill.classes(), ThemeAccent::Focus.classes())
                                >
                                    "New Note"
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                    WorkspaceDisplayState::NoNoteSelected => view! {
                        <div class="flex-1 flex items-center justify-center text-gray-300 dark:text-gray-700 select-none">
                            <div class="text-center">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-20 w-20 mx-auto opacity-20 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                </svg>
                                <p class="text-xl">No Note Selected</p>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn ToolbarButton(
    on_click: impl Fn(leptos::web_sys::MouseEvent) + Send + Sync + 'static,
    title: &'static str,
    aria_label: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            on:click=on_click
            title=title
            aria-label=aria_label
            class=move || format!("p-1.5 rounded {}", ThemeState::ToolbarButton.classes())
        >
            {children()}
        </button>
    }
}
