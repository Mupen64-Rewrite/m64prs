use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::WidgetExt;
use native::{NativeView, NativeViewAttributes, NativeViewKey, StackOrder};

pub mod native;

mod inner {
    use std::cell::{OnceCell, RefCell};

    use glib::subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::{ObjectSubclass, ObjectSubclassExt},
    };
    use gtk::subclass::widget::WidgetImpl;
    use gtk::{prelude::*, subclass::widget::WidgetImplExt};

    use crate::utils::dpi_conv;

    use super::native::NativeCompositor;

    #[derive(Default)]
    pub struct CompositorView {
        compositor: RefCell<Option<Box<dyn NativeCompositor>>>,
    }

    impl CompositorView {
        /// Executes a callback with the compositor.
        pub(super) fn with_compositor<F, T>(&self, f: F) -> Option<T>
        where
            F: FnOnce(&mut dyn NativeCompositor) -> T,
        {
            let mut compositor_ref = self.compositor.borrow_mut();
            if let Some(compositor) = compositor_ref.as_deref_mut() {
                Some(f(compositor))
            } else {
                None
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CompositorView {
        const NAME: &'static str = "M64PRS_CompositorView";
        type Type = super::CompositorView;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for CompositorView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_focusable(true);
            self.obj().set_focus_on_click(true);
        }
    }

    impl WidgetImpl for CompositorView {
        fn realize(&self) {
            self.parent_realize();
            let native = self
                .obj()
                .native()
                .expect("widget should be bound to a gtk::Native");
            let gdk_surface = native.surface().expect("gtk::Native should have a surface");

            log::info!("Window scale factor: {}", gdk_surface.scale());

            // compute position relative to the window's coordinate system
            let win_pos = self
                .obj()
                .compute_point(&native, &graphene::Point::zero())
                .unwrap();
            let physical_pos =
                dpi_conv::into_dpi_position(win_pos).to_physical(gdk_surface.scale() as f64);

            let mut compositor = self.compositor.borrow_mut();
            *compositor = Some(<dyn NativeCompositor>::new(gdk_surface, physical_pos));
        }

        fn unrealize(&self) {
            self.parent_unrealize();
            let mut compositor = self.compositor.borrow_mut();
            *compositor = None;
        }

        fn map(&self) {
            self.parent_map();
            self.with_compositor(|comp| comp.set_mapped(true));
        }

        fn unmap(&self) {
            self.parent_unmap();
            self.with_compositor(|comp| comp.set_mapped(false));
        }

        fn size_allocate(&self, _width: i32, _height: i32, _baseline: i32) {
            let native = self
                .obj()
                .native()
                .expect("widget should be bound to a gtk::Native");
            let gdk_surface = native.surface().expect("gtk::Native should have a surface");

            // compute position relative to the window's coordinate system
            let win_pos = self
                .obj()
                .compute_point(&native, &graphene::Point::zero())
                .unwrap();

            self.with_compositor(|comp| {
                let physical_pos = dpi_conv::into_dpi_position(win_pos)
                    .to_physical(comp.scale_factor().unwrap_or_else(|| gdk_surface.scale()));
                comp.set_position(physical_pos)
            });
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let native = self
                .obj()
                .native()
                .expect("widget should be bound to a gtk::Native");
            let gdk_surface = native.surface().expect("gtk::Native should have a surface");

            // compute bounds relative to GTK's coordinate system
            let gtk_bounds = self
                .with_compositor(|comp| {
                    let bounds = comp.total_bounds();
                    dpi_conv::into_graphene_size(bounds.to_logical::<f32>(
                        comp.scale_factor().unwrap_or_else(|| gdk_surface.scale()),
                    ))
                })
                .unwrap_or(graphene::Size::new(100.0, 100.0));
            log::debug!("bounds: {:?}", &gtk_bounds);

            let dimension: i32 = match orientation {
                gtk::Orientation::Horizontal => gtk_bounds.width(),
                gtk::Orientation::Vertical => gtk_bounds.height(),
                _ => unreachable!(),
            }
            .ceil() as i32;

            (dimension, dimension, -1, -1)
        }
    }
}

glib::wrapper! {
    pub struct CompositorView(ObjectSubclass<inner::CompositorView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for CompositorView {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

impl CompositorView {
    pub fn new_view(&self, attrs: NativeViewAttributes) -> Box<dyn NativeView> {
        let view = self
            .imp()
            .with_compositor(|comp| comp.new_view(attrs))
            .expect("compositor must be realized");

        self.queue_resize();
        view
    }
    pub fn del_view(&self, key: NativeViewKey) {
        self.imp()
            .with_compositor(|comp| comp.delete_view(key))
            .expect("compositor must be realized");
        self.queue_resize();
    }
    pub fn set_view_bounds(
        &self,
        key: NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {
        self.imp()
            .with_compositor(|comp| comp.set_view_bounds(key, position, size))
            .expect("compositor must be realized");
        self.queue_resize();
    }
    pub fn restack_view(&self, key: NativeViewKey, stack_order: StackOrder) {
        self.imp()
            .with_compositor(|comp| comp.restack_view(key, stack_order))
            .expect("compositor must be realized");
    }
}
