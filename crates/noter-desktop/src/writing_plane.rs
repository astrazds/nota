//! GTK writing-plane clamp: caps child allocation at a Pango-measured max width.
//!
//! GTK Stylesheet rejects CSS `max-width`, so this widget enforces the contract
//! measure in layout (left-aligned, grow up to N×`ch` then stop).

use std::cell::Cell;

use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct WritingPlane {
        pub max_width: Cell<i32>,
        pub child: glib::WeakRef<gtk::Widget>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WritingPlane {
        const NAME: &'static str = "NoterWritingPlane";
        type Type = super::WritingPlane;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for WritingPlane {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_css_classes(&["noter-writing-plane"]);
            // Fill cross-axis so vertical parents allocate full width; clamp in size_allocate.
            obj.set_halign(gtk::Align::Fill);
            obj.set_hexpand(true);
        }

        fn dispose(&self) {
            if let Some(child) = self.child.upgrade() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for WritingPlane {
        fn measure(
            &self,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let Some(child) = self.child.upgrade() else {
                return (0, 0, -1, -1);
            };
            let max_w = self.max_width.get();
            match orientation {
                gtk::Orientation::Horizontal => {
                    let (min, nat, min_baseline, nat_baseline) =
                        child.measure(orientation, for_size);
                    if max_w > 0 {
                        // Prefer the contract plane width so title/tags Entries expand
                        // to the full 72ch strip instead of their short natural size
                        // (which ellipsizes placeholders mid-word).
                        let min = min.min(max_w);
                        let nat = max_w.max(min);
                        (min, nat, min_baseline, nat_baseline)
                    } else {
                        (min, nat, min_baseline, nat_baseline)
                    }
                }
                _ => {
                    let width_for = if max_w <= 0 {
                        for_size
                    } else if for_size < 0 {
                        max_w
                    } else {
                        for_size.min(max_w)
                    };
                    child.measure(orientation, width_for)
                }
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let Some(child) = self.child.upgrade() else {
                return;
            };
            let max_w = self.max_width.get();
            let child_w = if max_w > 0 { width.min(max_w) } else { width };
            // Left-align within the allocated strip (parent may be full-bleed).
            child.allocate(child_w, height, baseline, None);
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if let Some(child) = self.child.upgrade() {
                self.obj().snapshot_child(&child, snapshot);
            }
        }
    }
}

glib::wrapper! {
    pub struct WritingPlane(ObjectSubclass<imp::WritingPlane>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WritingPlane {
    pub fn new(max_width: i32) -> Self {
        let plane: Self = glib::Object::new();
        plane.set_max_width(max_width);
        plane
    }

    pub fn set_max_width(&self, max_width: i32) {
        self.imp().max_width.set(max_width.max(1));
        self.queue_resize();
    }

    pub fn max_width(&self) -> i32 {
        self.imp().max_width.get()
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk::Widget>>) {
        if let Some(existing) = self.imp().child.upgrade() {
            existing.unparent();
        }
        if let Some(child) = child {
            let child = child.as_ref();
            child.set_parent(self);
            self.imp().child.set(Some(child));
        } else {
            self.imp().child.set(None);
        }
        self.queue_resize();
    }
}
