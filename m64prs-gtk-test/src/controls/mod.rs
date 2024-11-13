mod native;

mod inner {
    use std::{cell::RefCell, sync::Arc};

    use glib::subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::{FromObject, ObjectSubclass, ObjectSubclassExt},
    };
    use gtk::{
        graphene,
        prelude::*,
        subclass::widget::{WidgetImpl, WidgetImplExt},
    };

    use super::native::PlatformSubsurface;

    pub struct SubsurfaceContainer {
        subsurfaces: RefCell<Vec<Arc<dyn PlatformSubsurface>>>,
    }

    impl Default for SubsurfaceContainer {
        fn default() -> Self {
            Self {
                subsurfaces: RefCell::new(Vec::new()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubsurfaceContainer {
        const NAME: &'static str = "M64PRSChildWindowContainer";
        type Type = super::ChildWindowContainer;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for SubsurfaceContainer {}

    impl WidgetImpl for SubsurfaceContainer {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            let color = gdk::RGBA::new(1.0, 0.5, 0.0, 1.0);
            let rect = graphene::Rect::new(
                0.0,
                0.0,
                self.obj().width() as f32,
                self.obj().height() as f32,
            );

            snapshot.append_color(&color, &rect);
        }
    }
}

glib::wrapper! {
    pub struct ChildWindowContainer(ObjectSubclass<inner::SubsurfaceContainer>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ChildWindowContainer {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }
}
