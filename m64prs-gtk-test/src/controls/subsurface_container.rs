use std::{
    future::Future,
    sync::{Arc, Weak},
};

use gdk::prelude::SurfaceExt;
use glib::subclass::types::ObjectSubclassIsExt;
use graphene::{Point, Size};
use gtk::prelude::{NativeExt, WidgetExt};
use inner::SubsurfaceMetadata;
use pollster::FutureExt;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use relm4::RelmWidgetExt;
use send_wrapper::SendWrapper;

use super::native::{conv, PlatformSubsurface, SubsurfaceAttributes};

pub struct SubsurfaceHandle {
    subsurface: Arc<dyn PlatformSubsurface>,
    parent: SendWrapper<SubsurfaceContainer>,
}

impl SubsurfaceHandle {
    pub fn request_bounds(self, position: Option<Point>, size: Option<Size>) -> Self {
        glib::spawn_future(async move {
            self.parent.adjust_subsurface(&self, position, size);
            self
        })
        .block_on()
        .unwrap()
    }
}

impl HasDisplayHandle for SubsurfaceHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.subsurface.display_handle()
    }
}

impl HasWindowHandle for SubsurfaceHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.subsurface.window_handle()
    }
}

mod inner {
    use std::{cell::RefCell, sync::Arc};

    use glib::subclass::{
        object::ObjectImpl,
        types::{ObjectSubclass, ObjectSubclassExt},
    };
    use graphene::{Point, Rect, Size};
    use gtk::{
        graphene,
        prelude::*,
        subclass::widget::{WidgetImpl, WidgetImplExt},
    };

    use super::super::native::PlatformSubsurface;

    pub(super) struct SubsurfaceMetadata {
        pub(super) handle: Arc<dyn PlatformSubsurface>,
        pub(super) position: Point,
        pub(super) size: Size,
    }

    pub struct SubsurfaceContainer {
        pub(super) subsurfaces: RefCell<Vec<SubsurfaceMetadata>>,
    }

    impl Default for SubsurfaceContainer {
        fn default() -> Self {
            Self {
                subsurfaces: RefCell::new(Vec::new()),
            }
        }
    }

    impl Drop for SubsurfaceContainer {
        fn drop(&mut self) {
            let subsurfaces = self.subsurfaces.get_mut();
            assert!(
                subsurfaces
                    .iter()
                    .all(|s| Arc::strong_count(&s.handle) == 1),
                "subsurfaces should not outlive SubsurfaceContainer"
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubsurfaceContainer {
        const NAME: &'static str = "M64PRSChildWindowContainer";
        type Type = super::SubsurfaceContainer;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for SubsurfaceContainer {}

    impl WidgetImpl for SubsurfaceContainer {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.parent_measure(orientation, for_size)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            let color = gdk::RGBA::new(1.0, 0.5, 0.0, 1.0);
            let rect = Rect::new(
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
    /// A widget that contains subsurfaces.
    pub struct SubsurfaceContainer(ObjectSubclass<inner::SubsurfaceContainer>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SubsurfaceContainer {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn new_subsurface(
        &self,
        position: Point,
        size: Size,
        transparent: bool,
    ) -> SubsurfaceHandle {
        assert!(
            self.is_mapped(),
            "SubsurfaceContainer should be mapped to create subsurfaces"
        );
        let window = self
            .toplevel_window()
            .expect("SubsurfaceContainer should be attached to a window");
        let gdk_surface = window.surface().expect("gtk::Window should have a surface");
        let scale_factor = self.scale_factor() as f64;

        let g_position = self.compute_point(&window, &position).unwrap();

        let attributes = SubsurfaceAttributes::new()
            .with_position(conv::into_dpi_position(g_position).to_physical(scale_factor))
            .with_surface_size(conv::into_dpi_size(size).to_physical(scale_factor))
            .with_transparent(transparent);

        let handle: Arc<dyn PlatformSubsurface> =
            <dyn PlatformSubsurface>::new(&gdk_surface, attributes).into();

        let metadata = SubsurfaceMetadata {
            handle: Arc::clone(&handle),
            position,
            size,
        };
        self.imp().subsurfaces.borrow_mut().push(metadata);

        SubsurfaceHandle {
            subsurface: handle,
            parent: SendWrapper::new(self.clone()),
        }
    }

    fn adjust_subsurface(
        &self,
        handle: &SubsurfaceHandle,
        position: Option<Point>,
        size: Option<Size>,
    ) {
        assert!(
            &*handle.parent == self,
            "Cannot reposition subsurface from another controller!"
        );

        let window = self
            .toplevel_window()
            .expect("SubsurfaceContainer should be attached to a window");
        let scale_factor = self.scale_factor() as f64;

        let mut subsurfaces = self.imp().subsurfaces.borrow_mut();

        let entry = subsurfaces
            .iter_mut()
            .find(|entry| Arc::ptr_eq(&handle.subsurface, &entry.handle))
            .expect("handle should be valid");

        if let Some(position) = position {
            let g_position = self.compute_point(&window, &position).unwrap();
            entry
                .handle
                .set_position(conv::into_dpi_position(g_position).to_physical(scale_factor));
            entry.position = position;
        }
        if let Some(size) = size {
            entry
                .handle
                .set_size(conv::into_dpi_size(size).to_physical(scale_factor));
            entry.size = size;
        }
    }

    pub fn close_subsurface(&self, handle: SubsurfaceHandle) {
        assert!(
            &*handle.parent == self,
            "Cannot reposition subsurface from another controller!"
        );

        let mut subsurfaces = self.imp().subsurfaces.borrow_mut();
        let entry_idx = subsurfaces
            .iter()
            .position(|entry| Arc::ptr_eq(&handle.subsurface, &entry.handle))
            .expect("handle should be valid");
        subsurfaces.swap_remove(entry_idx);
    }
}
