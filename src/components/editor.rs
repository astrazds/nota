use crate::AppState;
use crate::NotificationTone;
use crate::backup::PendingBackupImport;
use crate::backup_controls::{
    cancel_pending_backup_import, confirm_pending_backup_import, import_backup_from_input_event,
};
use crate::components::CheatsheetModal;
use crate::editor_view::EditorViewMode;
use crate::model::Note;
use crate::note_workspace::{FocusIntent, WorkspaceDisplayState};
use crate::storage::SaveStatus;
use crate::tag_rules::{parse_tags_input, tags_to_input};
use crate::theme::{ThemeAccent, ThemeState, ThemeSurface, ThemeText};
use crate::ui_recipes;
use crate::writing_surface::{
    HIDDEN_BY_FILTER_MESSAGE, MarkdownCommand, WritingSurfaceModel, WritingSurfaceSelection,
    apply_formatting_command,
};
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, prelude::Closure};

const NOTIFICATION_HIDE_MS: i32 = 3_000;
type NotificationTimeout = Rc<RefCell<Option<(i32, Closure<dyn FnMut()>)>>>;

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("state not found");
    let show_cheatsheet = RwSignal::new(false);
    let recovery_pending_backup_import = RwSignal::new(None::<PendingBackupImport>);

    let title_input_ref = NodeRef::<leptos::html::Textarea>::new();
    let tags_input_ref = NodeRef::<leptos::html::Input>::new();
    let content_area_ref = NodeRef::<leptos::html::Textarea>::new();
    let tags_input_value = RwSignal::new(String::new());
    let is_editing_tags = RwSignal::new(false);

    let selected_note = Memo::new(move |_| state.selected_note());
    let footer_metrics = Memo::new(move |_| {
        selected_note
            .get()
            .map(|note| {
                (
                    note_line_count(&note.content),
                    note.word_count(),
                    note.character_count(),
                )
            })
            .unwrap_or_default()
    });
    let workspace_display_state = Memo::new(move |_| state.workspace_display_state());
    let selected_note_is_hidden_by_filter =
        Memo::new(move |_| state.selected_note_is_hidden_by_filter());
    let writing_surface_model = Memo::new(move |_| {
        selected_note.get().as_ref().map(|note| {
            WritingSurfaceModel::from_note(
                note,
                state.editor_view_mode.get(),
                selected_note_is_hidden_by_filter.get(),
            )
        })
    });
    let writing_model =
        Memo::new(move |_| writing_surface_model.get().and_then(|model| model.writing));
    let preview_model =
        Memo::new(move |_| writing_surface_model.get().and_then(|model| model.preview));
    let is_split_view = Memo::new(move |_| state.editor_view_mode.get() == EditorViewMode::Split);
    let preview_tags = Memo::new(move |_| {
        preview_model
            .get()
            .map(|preview| preview.tags)
            .unwrap_or_default()
    });
    let hidden_by_filter_message = Memo::new(move |_| {
        writing_surface_model
            .get()
            .and_then(|model| model.hidden_by_filter_message)
    });
    let previous_save_status = RwSignal::new(state.save_status.get_untracked());

    Effect::new(move |_| {
        let save_status = state.save_status.get();
        let previous = previous_save_status.get_untracked();
        if save_status == previous {
            return;
        }

        previous_save_status.set(save_status);
        match save_status {
            SaveStatus::Saving => {
                state.show_save_notification("Saving...", NotificationTone::Progress);
            }
            SaveStatus::Saved => {
                state.show_save_notification("Saved", NotificationTone::Success);
            }
        }
    });

    Effect::new(move |_| {
        if state.focus_intent() == FocusIntent::NoteTitle
            && let Some(input) = title_input_ref.get()
        {
            let _ = input.focus();
            state.take_focus_intent();
        }
    });

    Effect::new(move |_| {
        if is_editing_tags.get() {
            if let Some(input) = tags_input_ref.get() {
                let _ = input.focus();
            }
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

    let start_editing_tags = move |_| {
        is_editing_tags.set(true);
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

    let apply_format = move |command: MarkdownCommand| {
        if let Some(textarea) = content_area_ref.get() {
            let start_utf16 = textarea.selection_start().unwrap_or_default().unwrap_or(0);
            let end_utf16 = textarea.selection_end().unwrap_or_default().unwrap_or(0);
            let content = textarea.value();
            let formatted = apply_formatting_command(
                &content,
                WritingSurfaceSelection {
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
        <div
            data-testid="editor-frame"
            class=move || format!(
                "min-w-0 flex-1 flex flex-col h-full overflow-hidden relative transition-colors duration-300 {}",
                ThemeSurface::EditorChrome.classes()
            )
        >
            <CheatsheetModal show=show_cheatsheet />

            <div class="pointer-events-none absolute left-3 top-3 z-20 lg:hidden">
                <button
                    on:click=move |_| state.toggle_sidebar()
                    class=sidebar_toggle_button_classes
                    title="Toggle Sidebar"
                    aria-label="Toggle sidebar"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                    </svg>
                </button>
            </div>

            <div class="flex-1 flex overflow-hidden">
                {move || match workspace_display_state.get() {
                    WorkspaceDisplayState::StorageRecovery => view! {
                        <div class=move || format!("flex-1 flex items-center justify-center px-6 {}", ThemeState::EmptyState.classes())>
                            <div class="w-full max-w-md text-center">
                                <h2 class=ui_recipes::empty_state_title_text>"Saved Notes could not be loaded"</h2>
                                <p class=ui_recipes::empty_state_body_text>
                                    "Choose a recovery path to continue using Nota."
                                </p>
                                <div class="mt-6 flex flex-wrap items-center justify-center gap-2">
                                    <button
                                        type="button"
                                        disabled=move || !state.has_previous_snapshot()
                                        class=move || {
                                            let state_classes = if state.has_previous_snapshot() {
                                                ThemeAccent::PrimaryFill.classes().to_string()
                                            } else {
                                                "cursor-not-allowed bg-gray-200 text-gray-400 dark:bg-white/10 dark:text-gray-600".to_string()
                                            };
                                            format!("inline-flex min-h-10 items-center rounded-md px-4 py-2 transition-colors {} {state_classes}", ui_recipes::button_label_text())
                                        }
                                        on:click=move |_| state.restore_previous_snapshot()
                                    >
                                        "Restore previous snapshot"
                                    </button>
                                    <button
                                        type="button"
                                        class=move || format!("inline-flex min-h-10 items-center rounded-md px-4 py-2 transition-colors {} {}", ui_recipes::button_label_text(), ThemeState::SecondaryButton.classes())
                                        on:click=move |_| state.start_empty_after_storage_recovery()
                                    >
                                        "Start empty"
                                    </button>
                                    <label class=move || format!("inline-flex min-h-10 cursor-pointer items-center rounded-md px-4 py-2 transition-colors {} {}", ui_recipes::button_label_text(), ThemeState::SegmentedIdle.classes())>
                                        "Import Backup"
                                        <input
                                            type="file"
                                            accept="application/json,.json"
                                            class="sr-only"
                                            on:change=move |ev| {
                                                import_backup_from_input_event(
                                                    state,
                                                    recovery_pending_backup_import,
                                                    ev,
                                                );
                                            }
                                        />
                                    </label>
                                </div>
                                <Show when=move || recovery_pending_backup_import.get().is_some()>
                                    <div class=move || format!("mt-4 rounded-md border p-3 {} {}", ui_recipes::ui_body_text(), ThemeSurface::EditorChrome.classes())>
                                        {move || {
                                            recovery_pending_backup_import
                                                .get()
                                                .map(|pending| {
                                                    view! {
                                                        <div class="flex flex-wrap items-center justify-center gap-2">
                                                            <span class=move || ThemeText::Primary.classes()>
                                                                {format!(
                                                                    "Import {} notes: {} new, {} replace",
                                                                    pending.preview.total_imported_notes,
                                                                    pending.preview.notes_to_add,
                                                                    pending.preview.notes_to_replace
                                                                )}
                                                            </span>
                                                            <button
                                                                type="button"
                                                                class=move || format!("rounded-md px-3 py-1 {} {}", ui_recipes::button_label_text(), ThemeState::SegmentedIdle.classes())
                                                                on:click=move |_| {
                                                                    confirm_pending_backup_import(
                                                                        state,
                                                                        recovery_pending_backup_import,
                                                                    );
                                                                }
                                                            >
                                                                "Import"
                                                            </button>
                                                            <button
                                                                type="button"
                                                                class=move || format!("rounded-md px-3 py-1 {} {}", ui_recipes::button_label_text(), ThemeState::SegmentedIdle.classes())
                                                                on:click=move |_| {
                                                                    cancel_pending_backup_import(
                                                                        state,
                                                                        recovery_pending_backup_import,
                                                                    );
                                                                }
                                                            >
                                                                "Cancel"
                                                            </button>
                                                        </div>
                                                    }
                                                })
                                        }}
                                    </div>
                                </Show>
                            </div>
                        </div>
                    }.into_any(),
                    WorkspaceDisplayState::NoteSelected => view! {
                        <div class="flex-1 flex overflow-hidden divide-x divide-apple-gray-200 dark:divide-apple-dark-border">
                            <Show when=move || state.editor_view_mode.get().surfaces().writing>
                                <div class=move || format!("flex-1 flex flex-col overflow-hidden {}", ThemeSurface::WritingSurface.classes())>
                                    <Show when=move || hidden_by_filter_message.get().is_some()>
                                        <div class=hidden_filter_notice_classes>
                                            <p class=move || ThemeText::Muted.classes()>
                                                {move || hidden_by_filter_message.get().unwrap_or(HIDDEN_BY_FILTER_MESSAGE)}
                                            </p>
                                        </div>
                                    </Show>
                                    <div class=note_header_classes>
                                        <div class=note_header_stack_classes>
                                            <div class=note_title_row_classes>
                                                <textarea
                                                    node_ref=title_input_ref
                                                    rows="1"
                                                    class=move || note_title_textarea_classes(is_split_view.get())
                                                    placeholder="Note Title"
                                                    prop:value=move || writing_model.get().map(|note| note.title).unwrap_or_default()
                                                    on:input=on_input_title
                                                ></textarea>
                                                <Show when=move || is_editing_tags.get() || writing_model.get().is_none_or(|note| note.tags.is_empty())>
                                                    <input
                                                        node_ref=tags_input_ref
                                                        type="text"
                                                        class=tag_input_classes
                                                        placeholder="Add tags"
                                                        prop:value=move || tags_input_value.get()
                                                        on:focus=move |_| is_editing_tags.set(true)
                                                        on:input=on_input_tags
                                                        on:keydown=on_tags_keydown
                                                        on:blur=move |_| {
                                                            is_editing_tags.set(false);
                                                            commit_tags_input();
                                                        }
                                                    />
                                                </Show>
                                                <Show when=move || !is_editing_tags.get() && writing_model.get().is_some_and(|note| !note.tags.is_empty())>
                                                    <EditableTagList selected_note=selected_note on_edit=start_editing_tags />
                                                </Show>
                                            </div>
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
                                                                        class=move || format!("block w-full px-3 py-2 text-left transition-colors {} {}", ui_recipes::ui_body_text(), ThemeState::SegmentedIdle.classes())
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
                                                <details class=move || format!("max-w-xl rounded-md border p-3 {} {}", ui_recipes::ui_body_text(), ThemeSurface::EditorChrome.classes())>
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
                                                            class=move || format!("rounded-md px-3 py-1 {} {}", ui_recipes::button_label_text(), ThemeState::SegmentedIdle.classes())
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
                                    </div>
                                    <Show when=move || writing_model.get().is_some_and(|note| note.formatting_tools_visible)>
                                        <div class=formatting_tools_classes>
                                            <div class=formatting_tools_row_classes>
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
                                        </div>
                                    </Show>
                                    <div class=editor_body_frame_classes>
                                        <textarea
                                            node_ref=content_area_ref
                                            class=editor_body_textarea_classes
                                            placeholder="Start typing..."
                                            prop:value=move || writing_model.get().map(|note| note.content).unwrap_or_default()
                                            on:input=on_input_content
                                        ></textarea>
                                    </div>
                                </div>
                            </Show>

                            <Show when=move || state.editor_view_mode.get().surfaces().preview>
                                <div
                                    class=move || {
                                        preview_pane_classes(state.editor_view_mode.get() == EditorViewMode::Split)
                                    }
                                >
                                    <article class=preview_content_classes>
                                        <h1 class=preview_title_classes>{move || preview_model.get().map(|preview| preview.title).unwrap_or_default()}</h1>
                                        <PreviewTagList tags=preview_tags />
                                        <div inner_html=move || preview_model.get().map(|preview| preview.body_html).unwrap_or_default()></div>
                                    </article>
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
                                <h2 class=ui_recipes::empty_state_title_text>"Create your first note"</h2>
                                <p class=ui_recipes::empty_state_body_text>"Start with a title, then write in Markdown when you need it."</p>
                                <button
                                    on:click=move |_| state.create_note()
                                    class=move || format!("mt-6 inline-flex items-center rounded-md px-4 py-2 transition-colors {} {} {}", ui_recipes::button_label_text(), ThemeAccent::PrimaryFill.classes(), ThemeAccent::Focus.classes())
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
                                <p class=ui_recipes::empty_state_placeholder_text>No Note Selected</p>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>

            <div class=editor_area_footer_classes>
                <div class=editor_footer_stats_classes>
                    <span>{move || format!("Lines: {}", footer_metrics.get().0)}</span>
                    <span>{move || format!("Words: {}", footer_metrics.get().1)}</span>
                    <span>{move || format!("Characters: {}", footer_metrics.get().2)}</span>
                </div>
                <div class=editor_footer_mode_group_classes>
                    <span class=editor_footer_mode_label_classes>"Mode"</span>
                    <div class=editor_view_controls_classes>
                        <button
                            on:click=move |_| state.set_editor_view_mode(EditorViewMode::Write)
                            title="Write"
                            aria-label="Write mode"
                            aria-pressed=move || state.editor_view_mode.get() == EditorViewMode::Write
                            class=move || editor_view_button_classes(
                                state.editor_view_mode.get() == EditorViewMode::Write,
                                false,
                            )
                        >
                            "Write"
                        </button>
                        <button
                            on:click=move |_| state.set_editor_view_mode(EditorViewMode::Preview)
                            title="Preview"
                            aria-label="Preview mode"
                            aria-pressed=move || state.editor_view_mode.get() == EditorViewMode::Preview
                            class=move || editor_view_button_classes(
                                state.editor_view_mode.get() == EditorViewMode::Preview,
                                false,
                            )
                        >
                            "Preview"
                        </button>
                        <button
                            on:click=move |_| state.set_editor_view_mode(EditorViewMode::Split)
                            title="Split"
                            aria-label="Split mode"
                            aria-pressed=move || state.editor_view_mode.get() == EditorViewMode::Split
                            class=move || editor_view_button_classes(
                                state.editor_view_mode.get() == EditorViewMode::Split,
                                true,
                            )
                        >
                            "Split"
                        </button>
                        <button
                            on:click=move |_| show_cheatsheet.set(true)
                            title="Markdown Help"
                            aria-label="Show markdown cheatsheet"
                            class=markdown_help_button_classes
                        >
                            <span class="md:hidden">"?"</span>
                            <span class="hidden md:inline">"Help"</span>
                        </button>
                    </div>
                </div>
                <div class=editor_footer_spacer_classes aria-hidden="true"></div>
            </div>
        </div>
    }
}

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

fn editor_view_button_classes(is_active: bool, is_split: bool) -> String {
    ui_recipes::compact_segmented_button(is_active, is_split)
}

fn sidebar_toggle_button_classes() -> String {
    format!(
        "pointer-events-auto inline-flex h-11 w-11 items-center justify-center rounded-md shadow-sm lg:hidden {} {}",
        ThemeSurface::EditorChrome.classes(),
        ThemeAccent::PrimaryText.classes()
    )
}

fn editor_area_footer_classes() -> String {
    ui_recipes::editor_footer()
}

fn editor_view_controls_classes() -> &'static str {
    ui_recipes::compact_controls()
}

fn editor_footer_stats_classes() -> String {
    ui_recipes::editor_footer_stats()
}

fn editor_footer_mode_group_classes() -> &'static str {
    ui_recipes::editor_footer_mode_group()
}

fn editor_footer_mode_label_classes() -> String {
    ui_recipes::editor_footer_mode_label()
}

fn editor_footer_spacer_classes() -> &'static str {
    ui_recipes::editor_footer_spacer()
}

fn note_line_count(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.lines().count()
    }
}

fn formatting_tools_classes() -> String {
    format!(
        "border-y border-apple-notebook-border py-1.5 dark:border-apple-notebook-darkBorder {} {}",
        ui_recipes::pane_inline_inset(),
        ThemeSurface::EditorChrome.classes()
    )
}

fn formatting_tools_row_classes() -> String {
    format!("flex items-center gap-1 {}", ui_recipes::note_measure())
}

fn formatting_tool_button_classes() -> String {
    format!(
        "inline-flex h-11 min-w-[2.75rem] items-center justify-center rounded-md px-2.5 py-2 md:h-6 md:min-w-0 md:p-1 {}",
        ThemeState::ToolbarButton.classes()
    )
}

fn markdown_help_button_classes() -> String {
    ui_recipes::compact_help_button()
}

fn hidden_filter_notice_classes() -> String {
    format!(
        "mx-6 mt-5 max-w-[72ch] rounded-md border px-3 py-2 md:mx-8 {} {}",
        ui_recipes::ui_body_text(),
        ThemeSurface::EditorChrome.classes()
    )
}

fn note_header_classes() -> &'static str {
    "border-b border-transparent pb-2 pl-20 pr-6 pt-4 md:px-8 md:pt-5"
}

fn note_header_stack_classes() -> String {
    format!("space-y-2 {}", ui_recipes::note_measure())
}

fn note_title_row_classes() -> &'static str {
    "flex min-w-0 flex-col gap-2 sm:flex-row sm:items-start sm:gap-5"
}

fn note_title_textarea_classes(_is_split: bool) -> String {
    format!(
        "w-full min-w-0 resize-none overflow-hidden break-words whitespace-pre-wrap [field-sizing:content] sm:w-auto sm:max-w-[28rem] {} focus:outline-none bg-transparent {} {}",
        ui_recipes::note_title_text(),
        ThemeText::Primary.classes(),
        ThemeText::Placeholder.classes()
    )
}

fn preview_title_classes() -> String {
    format!("mb-4 {}", ui_recipes::preview_title_text())
}

fn tag_input_classes() -> String {
    format!(
        "h-7 w-full min-w-0 max-w-xs bg-transparent px-0 py-1 focus:outline-none placeholder:text-gray-400 dark:placeholder:text-gray-600 sm:mt-0.5 {} {}",
        ui_recipes::ui_control_text(),
        ThemeText::Muted.classes()
    )
}

fn edit_tags_button_classes() -> String {
    format!(
        "inline-flex h-11 items-center rounded-md px-3 md:h-auto md:px-2 md:py-0.5 {} {}",
        ui_recipes::ui_control_text(),
        ThemeState::SegmentedIdle.classes()
    )
}

fn editor_body_frame_classes() -> String {
    format!(
        "flex-1 min-h-0 pb-8 pt-4 {}",
        ui_recipes::pane_inline_inset()
    )
}

fn editor_body_textarea_classes() -> String {
    format!(
        "block h-full focus:outline-none resize-none bg-transparent dark:text-gray-300 {} {} {}",
        ui_recipes::note_measure(),
        ui_recipes::editor_body_text(),
        ThemeAccent::Selection.classes()
    )
}

fn preview_pane_classes(is_split: bool) -> String {
    if is_split {
        format!(
            "flex-1 overflow-y-auto pb-8 shadow-inner border-l transition-colors {} {} {}",
            ui_recipes::pane_inline_inset(),
            ui_recipes::pane_top_inset(),
            ThemeSurface::SplitPreview.classes()
        )
    } else {
        format!(
            "flex-1 overflow-y-auto pb-8 transition-colors {} {} {}",
            ui_recipes::pane_inline_inset(),
            ui_recipes::pane_top_inset(),
            ThemeSurface::Preview.classes()
        )
    }
}

fn preview_content_classes() -> &'static str {
    ui_recipes::preview_body_text()
}

#[component]
fn EditableTagList(
    selected_note: Memo<Option<Note>>,
    on_edit: impl Fn(leptos::web_sys::MouseEvent) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex max-w-xl flex-wrap items-center gap-1.5">
            {move || selected_note
                .get()
                .map(|note| view! {
                    {note.tags
                        .into_iter()
                        .map(|tag| {
                            let tag_for_remove = tag.clone();
                            view! {
                                <span
                                    class=move || format!("inline-flex items-center {}", ui_recipes::tag_pill())
                                    title=format!("Tag {tag_for_remove}")
                                >
                                    {format!("#{tag}")}
                                </span>
                            }
                        })
                        .collect_view()}
                    <button
                        type="button"
                        class=edit_tags_button_classes
                        on:click=on_edit
                    >
                        "Edit tags"
                    </button>
                })
            }
        </div>
    }
}

#[component]
fn PreviewTagList(tags: Memo<Vec<String>>) -> impl IntoView {
    view! {
        <Show when=move || !tags.get().is_empty()>
            <div class="not-prose mb-5 flex flex-wrap gap-1.5">
                {move || tags
                    .get()
                    .into_iter()
                    .map(|tag| {
                        view! {
                            <span class=move || format!("inline-flex items-center {}", ui_recipes::tag_pill())>
                                {format!("#{tag}")}
                            </span>
                        }
                    })
                    .collect_view()
                }
            </div>
        </Show>
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
            class=formatting_tool_button_classes
        >
            {children()}
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_editor_view_controls_keep_touch_sized_targets() {
        let sidebar = sidebar_toggle_button_classes();
        let footer = editor_area_footer_classes();
        let controls = editor_view_controls_classes();
        let write = editor_view_button_classes(false, false);
        let preview = editor_view_button_classes(true, false);
        let help = markdown_help_button_classes();

        assert!(footer.contains("h-[45px]"));
        assert!(footer.contains("px-4"));
        assert!(footer.contains("py-1.5"));
        assert!(footer.contains("text-[11px]"));
        assert!(footer.contains("leading-4"));
        assert!(footer.contains("border-t"));
        assert!(footer.contains("border-apple-notebook-border"));
        assert!(footer.contains("justify-between"));
        assert!(controls.contains("flex"));
        assert!(controls.contains("gap-x-1"));
        assert!(!controls.contains("gap-y-1"));
        assert!(sidebar.contains("h-11"));
        assert!(sidebar.contains("w-11"));
        assert!(sidebar.contains("lg:hidden"));
        assert!(write.contains("inline-flex"));
        for classes in [&write, &preview, &help] {
            assert!(classes.contains("h-11"));
            assert!(classes.contains("md:h-6"));
            assert!(classes.contains("md:px-2"));
            assert!(classes.contains("md:py-0"));
            assert!(classes.contains("md:text-[11px]"));
        }
        assert!(write.contains("min-w-[2.75rem]"));
        assert!(preview.contains("min-w-[2.75rem]"));
        assert!(help.contains("w-11"));
    }

    #[test]
    fn editor_footer_matches_the_notebook_status_bar_structure() {
        let stats = editor_footer_stats_classes();
        let mode_group = editor_footer_mode_group_classes();
        let mode_label = editor_footer_mode_label_classes();
        let spacer = editor_footer_spacer_classes();

        assert!(stats.contains("sm:flex"));
        assert!(stats.contains("gap-4"));
        assert!(mode_group.contains("justify-center"));
        assert!(mode_label.contains("sm:inline"));
        assert!(spacer.contains("flex-1"));
        assert!(spacer.contains("sm:block"));
        assert_eq!(note_line_count(""), 0);
        assert_eq!(note_line_count("One"), 1);
        assert_eq!(note_line_count("One\nTwo\nThree"), 3);
    }

    #[test]
    fn split_mode_control_stays_desktop_only() {
        let split = editor_view_button_classes(false, true);

        assert!(split.contains("hidden"));
        assert!(split.contains("lg:inline-flex"));
        assert!(split.starts_with("hidden lg:inline-flex"));
        assert!(!split.starts_with("inline-flex"));
    }

    #[test]
    fn formatting_tools_live_inside_the_writing_surface() {
        let toolbar = formatting_tools_classes();
        let toolbar_row = formatting_tools_row_classes();
        let button = formatting_tool_button_classes();

        assert!(toolbar.contains("border-y"));
        assert!(toolbar.contains("px-6"));
        assert!(toolbar.contains("md:px-8"));
        assert!(!toolbar.contains("sticky"));
        assert!(toolbar_row.contains("flex"));
        assert!(toolbar_row.contains("gap-1"));
        assert!(toolbar_row.contains("max-w-[72ch]"));
        assert!(button.contains("rounded"));
        assert!(button.contains("h-11"));
        assert!(button.contains("min-w-[2.75rem]"));
        assert!(button.contains("md:p-1"));
    }

    #[test]
    fn note_header_and_filter_notice_share_the_editor_measure() {
        let notice = hidden_filter_notice_classes();
        let header = note_header_classes();
        let header_stack = note_header_stack_classes();
        let header_row = note_title_row_classes();

        assert!(notice.contains("max-w-[72ch]"));
        assert!(notice.contains("mx-6"));
        assert!(notice.contains("md:mx-8"));
        assert!(header.contains("pl-20"));
        assert!(header.contains("md:px-8"));
        assert!(header.contains("pt-4"));
        assert!(header_stack.contains("space-y-2"));
        assert!(header_stack.contains("max-w-[72ch]"));
        assert!(header_row.contains("sm:flex-row"));
        assert!(header_row.contains("sm:gap-5"));
    }

    #[test]
    fn note_title_editor_wraps_long_titles_without_horizontal_overflow() {
        let title = note_title_textarea_classes(false);

        assert!(title.contains("w-full"));
        assert!(title.contains("min-w-0"));
        assert!(title.contains("break-words"));
        assert!(title.contains("whitespace-pre-wrap"));
        assert!(title.contains("[field-sizing:content]"));
        assert!(title.contains("overflow-hidden"));
        assert!(title.contains("resize-none"));
    }

    #[test]
    fn split_title_editor_keeps_the_same_note_header_scale_as_single_pane() {
        let single = note_title_textarea_classes(false);
        let split = note_title_textarea_classes(true);
        let preview = preview_title_classes();

        assert!(single.contains("text-xl"));
        assert!(single.contains("md:text-2xl"));
        assert!(split.contains("text-xl"));
        assert!(split.contains("md:text-2xl"));
        assert!(preview.contains("text-xl"));
        assert!(preview.contains("md:text-2xl"));
        assert!(preview.contains("mb-4"));
    }

    #[test]
    fn edit_tags_button_keeps_mobile_touch_target_and_desktop_density() {
        let button = edit_tags_button_classes();

        assert!(button.contains("h-11"));
        assert!(button.contains("md:h-auto"));
        assert!(button.contains("md:px-2"));
        assert!(button.contains("md:py-0.5"));
    }

    #[test]
    fn editor_body_text_matches_preview_scale() {
        let editor_body = editor_body_textarea_classes();
        let preview_pane = preview_pane_classes(false);
        let preview_content = preview_content_classes();

        assert!(editor_body.contains("text-sm"));
        assert!(editor_body.contains("max-w-[72ch]"));
        assert!(!editor_body.contains("text-base"));
        assert!(!preview_pane.contains("prose-base"));
        assert!(preview_pane.contains("md:px-8"));
        assert!(preview_pane.contains("md:pt-5"));
        assert!(preview_content.contains("prose-sm"));
        assert!(preview_content.contains("max-w-[72ch]"));
        assert!(!preview_content.contains("mx-auto"));
    }

    #[test]
    fn split_preview_uses_the_same_body_scale_as_preview() {
        let split_preview = preview_pane_classes(true);
        let preview_content = preview_content_classes();

        assert!(split_preview.contains("shadow-inner"));
        assert!(split_preview.contains("md:px-8"));
        assert!(split_preview.contains("md:pt-5"));
        assert!(preview_content.contains("prose-sm"));
        assert!(preview_content.contains("max-w-[72ch]"));
        assert!(!preview_content.contains("mx-auto"));
    }

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
