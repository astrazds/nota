use nota_core::note_discovery::HighlightSegment;
use nota_desktop::app::{AppModel, AppMsg};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::gtk::prelude::*;
use relm4::{RelmWidgetExt, gtk};
use uuid::Uuid;

pub(super) struct NoteLists {
    note_rows: FactoryVecDeque<NoteRow>,
    deleted_rows: FactoryVecDeque<DeletedRow>,
}

impl NoteLists {
    pub(super) fn new(sender: &relm4::Sender<AppMsg>) -> Self {
        let note_rows_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        note_rows_container.set_css_classes(&["nota-note-list"]);
        let note_rows = FactoryVecDeque::builder()
            .launch(note_rows_container)
            .forward(sender, |output| match output {
                NoteRowOutput::Select(id) => AppMsg::SelectNote(id),
                NoteRowOutput::TogglePin(id) => AppMsg::TogglePin(id),
                NoteRowOutput::Delete(id) => AppMsg::RequestDelete(id),
                NoteRowOutput::SelectTag(tag) => AppMsg::SelectTag(tag),
            });
        let deleted_rows_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let deleted_rows = FactoryVecDeque::builder()
            .launch(deleted_rows_container)
            .forward(sender, |output| match output {
                DeletedRowOutput::Restore(id) => AppMsg::RestoreRecentlyDeleted(id),
                DeletedRowOutput::Clear(id) => AppMsg::PermanentlyDelete(id),
            });
        Self {
            note_rows,
            deleted_rows,
        }
    }

    pub(super) fn notes_widget(&self) -> &gtk::Box {
        self.note_rows.widget()
    }
    pub(super) fn deleted_widget(&self) -> &gtk::Box {
        self.deleted_rows.widget()
    }

    pub(super) fn refresh(&mut self, app: &AppModel) {
        {
            let dark = matches!(app.theme, nota_core::transition::ThemePreference::Dark);
            let next: Vec<NoteRow> = app
                .note_list_render_model()
                .projection
                .rows
                .into_iter()
                .map(|row| NoteRow {
                    id: row.id,
                    title_markup: markup_or_plain(&row.title_highlights, &row.display_title),
                    date: row.display_date,
                    preview_markup: markup_or_plain(&row.preview_highlights, &row.preview),
                    tags: row.tags,
                    tag_highlights: row.tag_highlights,
                    pinned: row.is_pinned,
                    selected: row.is_selected,
                    dark,
                })
                .collect();
            let mut rows = self.note_rows.guard();
            let update_in_place = note_list_can_update_in_place(
                &(0..rows.len())
                    .filter_map(|index| rows.get(index).map(|row| row.id))
                    .collect::<Vec<_>>(),
                &next.iter().map(|row| row.id).collect::<Vec<_>>(),
            );
            if update_in_place {
                // Keep the existing row widgets so the sidebar does not jump to the
                // top or steal focus when the user selects a Note.
                for (index, row) in next.into_iter().enumerate() {
                    if let Some(current) = rows.get_mut(index) {
                        *current = row;
                    }
                }
            } else {
                rows.clear();
                for row in next {
                    rows.push_back(row);
                }
            }
        }
        {
            let mut rows = self.deleted_rows.guard();
            rows.clear();
            for note in app.workspace.recently_deleted_notes() {
                rows.push_back(DeletedRow {
                    id: note.id,
                    title: note.display_title().to_string(),
                });
            }
        }
    }
}

#[derive(Debug, Clone)]
struct NoteRow {
    id: Uuid,
    title_markup: String,
    date: String,
    preview_markup: String,
    tags: Vec<String>,
    tag_highlights: Vec<Vec<HighlightSegment>>,
    pinned: bool,
    selected: bool,
    /// Propagated onto the GTK popover so absolute theme tokens can match dark mode.
    dark: bool,
}

#[derive(Debug)]
enum NoteRowOutput {
    Select(Uuid),
    TogglePin(Uuid),
    Delete(Uuid),
    SelectTag(String),
}

#[relm4::factory]
impl FactoryComponent for NoteRow {
    type Init = Self;
    type Input = ();
    type Output = NoteRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 0,
            set_css_classes: &["nota-note-row"],
            #[watch]
            set_class_active: ("selected", self.selected),

            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                set_css_classes: &["nota-note-select"],
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(NoteRowOutput::Select(id));
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 3,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,

                        gtk::Label {
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            set_xalign: 0.0,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_css_classes: &["nota-note-title"],
                            set_use_markup: true,
                            #[watch]
                            set_label: &self.title_markup,
                        },
                        gtk::Label {
                            set_css_classes: &["nota-note-pin"],
                            #[watch]
                            set_label: if self.pinned { "◆" } else { "" },
                        },
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 7,

                        gtk::Label {
                            set_css_classes: &["nota-note-date"],
                            #[watch]
                            set_label: &self.date,
                        },
                        gtk::Label {
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            set_xalign: 0.0,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_css_classes: &["nota-note-preview"],
                            set_use_markup: true,
                            #[watch]
                            set_label: &self.preview_markup,
                        },
                    },
                    #[name(tag_list)]
                    gtk::FlowBox {
                        set_css_classes: &["nota-note-tag-list"],
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                        set_selection_mode: gtk::SelectionMode::None,
                        set_homogeneous: false,
                        set_column_spacing: 4,
                        set_row_spacing: 4,
                        #[watch]
                        set_visible: !self.tags.is_empty(),
                    },
                },
            },

            gtk::MenuButton {
                set_icon_name: "view-more-symbolic",
                set_tooltip_text: Some("Note actions"),
                set_css_classes: &["nota-note-actions"],

                #[wrap(Some)]
                set_popover: popover = &gtk::Popover {
                    set_position: gtk::PositionType::Bottom,
                    #[watch]
                    set_css_classes: if self.dark {
                        &["nota-note-actions-popover", "nota-dark"]
                    } else {
                        &["nota-note-actions-popover"]
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        #[watch]
                        set_css_classes: if self.dark {
                            &["nota-note-menu", "nota-dark"]
                        } else {
                            &["nota-note-menu"]
                        },

                        gtk::Button {
                            set_halign: gtk::Align::Fill,
                            #[watch]
                            set_css_classes: if self.dark {
                                &["nota-note-menu-item", "nota-dark"]
                            } else {
                                &["nota-note-menu-item"]
                            },
                            #[watch]
                            set_label: if self.pinned { "Unpin" } else { "Pin" },
                            connect_clicked[sender, id = self.id] => move |_| {
                                let _send_result = sender.output(NoteRowOutput::TogglePin(id));
                            },
                        },
                        gtk::Button {
                            set_halign: gtk::Align::Fill,
                            set_label: "Delete",
                            #[watch]
                            set_css_classes: if self.dark {
                                &["nota-note-menu-item", "destructive-action", "nota-dark"]
                            } else {
                                &["nota-note-menu-item", "destructive-action"]
                            },
                            connect_clicked[sender, id = self.id] => move |_| {
                                let _send_result = sender.output(NoteRowOutput::Delete(id));
                            },
                        },
                    },
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        sync_tag_buttons(&widgets.tag_list, &self.tags, &self.tag_highlights, &sender);
        widgets
    }

    fn post_view() {
        sync_tag_buttons(tag_list, &self.tags, &self.tag_highlights, &sender);
    }
}

#[derive(Debug, Clone)]
struct DeletedRow {
    id: Uuid,
    title: String,
}

#[derive(Debug)]
enum DeletedRowOutput {
    Restore(Uuid),
    Clear(Uuid),
}

#[relm4::factory]
impl FactoryComponent for DeletedRow {
    type Init = Self;
    type Input = ();
    type Output = DeletedRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_css_classes: &["nota-deleted-row"],

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
                #[watch]
                set_label: &self.title,
            },
            gtk::Button {
                set_label: "Restore",
                set_css_classes: &["nota-small-button"],
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(DeletedRowOutput::Restore(id));
                },
            },
            gtk::Button {
                set_label: "Delete",
                set_css_classes: &["nota-small-button", "danger"],
                connect_clicked[sender, id = self.id] => move |_| {
                    let _send_result = sender.output(DeletedRowOutput::Clear(id));
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
}

fn glib_escape(text: &str) -> String {
    gtk::glib::markup_escape_text(text).to_string()
}

fn highlight_markup(segments: &[HighlightSegment]) -> String {
    segments
        .iter()
        .map(|segment| {
            let text = gtk::glib::markup_escape_text(&segment.text);
            if segment.is_match {
                format!("<span background=\"#FFB340\" foreground=\"#25221F\">{text}</span>")
            } else {
                text.to_string()
            }
        })
        .collect()
}

fn markup_or_plain(segments: &[HighlightSegment], plain: &str) -> String {
    if segments.is_empty() {
        glib_escape(plain)
    } else {
        highlight_markup(segments)
    }
}

fn sync_tag_buttons(
    tag_list: &gtk::FlowBox,
    tags: &[String],
    tag_highlights: &[Vec<HighlightSegment>],
    sender: &FactorySender<NoteRow>,
) {
    let tags_unchanged = tags.iter().enumerate().all(|(index, tag)| {
        let Ok(position) = i32::try_from(index) else {
            return false;
        };
        tag_list
            .child_at_index(position)
            .and_then(|child| child.child())
            .and_then(|child| child.downcast::<gtk::Button>().ok())
            .and_then(|button| button.child())
            .and_then(|child| child.downcast::<gtk::Label>().ok())
            .is_some_and(|label| label.text() == format!("#{tag}"))
    }) && i32::try_from(tags.len())
        .ok()
        .is_some_and(|position| tag_list.child_at_index(position).is_none());

    if tags_unchanged {
        for (index, tag) in tags.iter().enumerate() {
            let Some(label) = i32::try_from(index)
                .ok()
                .and_then(|position| tag_list.child_at_index(position))
                .and_then(|child| child.child())
                .and_then(|child| child.downcast::<gtk::Button>().ok())
                .and_then(|button| button.child())
                .and_then(|child| child.downcast::<gtk::Label>().ok())
            else {
                continue;
            };
            let highlights = tag_highlights
                .get(index)
                .map(Vec::as_slice)
                .unwrap_or_default();
            label.set_markup(&format!("#{}", markup_or_plain(highlights, tag)));
        }
        return;
    }

    while let Some(child) = tag_list.first_child() {
        tag_list.remove(&child);
    }

    for (index, tag) in tags.iter().enumerate() {
        let highlights = tag_highlights
            .get(index)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let label = gtk::Label::new(None);
        label.set_markup(&format!("#{}", markup_or_plain(highlights, tag)));

        let button = gtk::Button::new();
        button.set_css_classes(&["nota-note-tags"]);
        button.set_child(Some(&label));
        let accessible_label = format!("Filter by tag {tag}");
        button.update_property(&[gtk::accessible::Property::Label(&accessible_label)]);
        button.set_tooltip_text(Some(&accessible_label));

        let selected_tag = tag.clone();
        let sender = sender.clone();
        button.connect_clicked(move |_| {
            let _send_result = sender.output(NoteRowOutput::SelectTag(selected_tag.clone()));
        });
        tag_list.append(&button);
    }
}

fn note_list_can_update_in_place(current_ids: &[Uuid], next_ids: &[Uuid]) -> bool {
    current_ids == next_ids
}

#[cfg(test)]
mod note_list_sync_tests {
    use super::note_list_can_update_in_place;
    use uuid::Uuid;

    #[test]
    fn selecting_a_note_keeps_row_identity_so_widgets_can_stay_mounted() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        assert!(note_list_can_update_in_place(
            &[first, second],
            &[first, second]
        ));
    }

    #[test]
    fn filtering_or_reordering_rebuilds_the_note_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        assert!(!note_list_can_update_in_place(&[first, second], &[second]));
        assert!(!note_list_can_update_in_place(
            &[first, second],
            &[second, first]
        ));
    }
}
