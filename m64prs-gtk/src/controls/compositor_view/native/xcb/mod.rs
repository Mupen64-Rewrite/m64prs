use std::{num::NonZero, ptr::NonNull, sync::Arc};

use gdk::prelude::*;
use glutin::display::DisplayApiPreference;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle, XcbDisplayHandle,
    XcbWindowHandle,
};
use slotmap::DenseSlotMap;
use state::{DisplayState, ScopeGuard, X11DisplayExt};
use x11rb::protocol::{
    shape,
    xfixes::ConnectionExt as XFixesConnectionExt,
    xproto::{self, ConfigureWindowAux, ConnectionExt, CreateWindowAux},
};

use super::{NativeCompositor, NativeView, NativeViewAttributes, NativeViewKey};

mod state;

pub struct XcbCompositor {
    display_state: Arc<DisplayState>,
    views: DenseSlotMap<NativeViewKey, XcbView>,

    current_bounds: dpi::PhysicalSize<u32>,
    mapped: bool,

    window: u32,
    visual_id: u32,
}

struct XcbView {
    display_state: Arc<DisplayState>,
    window: u32,
    visual_id: u32,

    position: dpi::PhysicalPosition<i32>,
    size: dpi::PhysicalSize<u32>,
    transparent: bool,
}

struct XcbViewHandle {
    view_key: NativeViewKey,
    display_state: Arc<DisplayState>,
    window: u32,
    visual_id: u32,
}

impl XcbCompositor {
    pub(super) fn new(
        gdk_surface: &gdk_x11::X11Surface,
        position: dpi::PhysicalPosition<i32>,
    ) -> Self {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_x11::X11Display>()
            .unwrap();
        let st = gdk_display.display_state();

        let parent_window: u32 = gdk_surface
            .xid()
            .try_into()
            .expect("XID should fit into u32");

        let (depth, visual_id) = st.find_depth_and_visual(false);

        let colormap = st.request_with_new_id(|id, conn| {
            conn.create_colormap(xproto::ColormapAlloc::NONE, id, st.root_window(), visual_id)
        });

        let window = st.request_with_new_id(|id, conn| {
            let aux = CreateWindowAux::new()
                .event_mask(xproto::EventMask::EXPOSURE)
                .override_redirect(1)
                .border_pixel(0x00000000)
                .background_pixel(0xFF000000)
                .colormap(colormap);
            conn.create_window(
                depth,
                id,
                parent_window,
                0,
                0,
                1,
                1,
                0,
                xproto::WindowClass::INPUT_OUTPUT,
                visual_id,
                &aux,
            )
        });

        st.request_void(|conn| {
            let aux = ConfigureWindowAux::new().x(position.x).y(position.y);
            conn.configure_window(window, &aux)
        });

        {
            let (ver_major, _) = st.init_xfixes();
            debug_assert!(ver_major >= 2);

            let region = st.request_with_new_id(|id, conn| conn.xfixes_create_region(id, &[]));
            let _guard = ScopeGuard::new(|| {
                let _ = st.conn.xfixes_destroy_region(region);
            });

            st.request_void(|conn| {
                conn.xfixes_set_window_shape_region(window, shape::SK::INPUT, 0, 0, region)
            });
        }

        Self {
            display_state: st,
            views: DenseSlotMap::with_key(),

            current_bounds: dpi::PhysicalSize::new(0, 0),
            mapped: false,

            window,
            visual_id,
        }
    }

    fn recompute_bounds(&mut self) {
        let (max_w, max_h) = self
            .views
            .iter()
            .fold((0u32, 0u32), |(max_w, max_h), (_, view)| {
                let max_w = u32::max(max_w, (view.position.x + view.size.width as i32) as u32);
                let max_h = u32::max(max_h, (view.position.y + view.size.height as i32) as u32);
                (max_w, max_h)
            });
        self.current_bounds = dpi::PhysicalSize::new(max_w, max_h);
        self.on_bounds_changed();
    }

    fn on_bounds_changed(&mut self) {
        log::info!("updating bounds to {:?}", self.current_bounds);

        let st = &*self.display_state;

        st.request_void(|conn| {
            let aux = ConfigureWindowAux::new()
                .width(self.current_bounds.width.max(1))
                .height(self.current_bounds.height.max(1));

            conn.configure_window(self.window, &aux)
        });
    }
}

impl XcbView {}

impl NativeCompositor for XcbCompositor {
    fn new_view(&mut self, attrs: NativeViewAttributes) -> Box<dyn super::NativeView> {
        let st = &self.display_state;

        let size: dpi::PhysicalSize<u32> = attrs.surface_size;
        let position: dpi::PhysicalPosition<i32> = attrs.position;
        let transparent: bool = attrs.transparent;

        let (depth, visual_id) = st.find_depth_and_visual(transparent);

        let colormap = st.request_with_new_id(|id, conn| {
            conn.create_colormap(xproto::ColormapAlloc::NONE, id, st.root_window(), visual_id)
        });

        let parent_window = self.window;

        let window = st.request_with_new_id(|id, conn| {
            let aux = CreateWindowAux::new()
                .event_mask(xproto::EventMask::EXPOSURE)
                .override_redirect(1)
                .border_pixel(0x00000000)
                .colormap(colormap);
            conn.create_window(
                depth,
                id,
                parent_window,
                0,
                0,
                1,
                1,
                0,
                xproto::WindowClass::INPUT_OUTPUT,
                visual_id,
                &aux,
            )
        });

        st.request_void(|conn| {
            let aux = ConfigureWindowAux::new()
                .x(position.x)
                .y(position.y)
                .width(size.width as u32)
                .height(size.height as u32);
            conn.configure_window(window, &aux)
        });

        {
            let (ver_major, _) = st.init_xfixes();
            debug_assert!(ver_major >= 2);

            let region = st.request_with_new_id(|id, conn| conn.xfixes_create_region(id, &[]));
            let _guard = ScopeGuard::new(|| {
                let _ = st.conn.xfixes_destroy_region(region);
            });

            st.request_void(|conn| {
                conn.xfixes_set_window_shape_region(window, shape::SK::INPUT, 0, 0, region)
            });
        }

        st.request_void(|conn| conn.map_window(window));

        let view = XcbView {
            display_state: Arc::clone(&self.display_state),
            window,
            visual_id,
            position,
            size,
            transparent,
        };
        let view_key = self.views.insert(view);

        self.recompute_bounds();

        Box::new(XcbViewHandle {
            view_key,
            display_state: Arc::clone(&self.display_state),
            window,
            visual_id,
        })
    }

    fn delete_view(&mut self, view_key: super::NativeViewKey) {
        if self.views.remove(view_key).is_none() {
            panic!("delete_view should be called with a valid key")
        };

        // recompute bounds
        self.recompute_bounds();
    }

    fn set_view_bounds(
        &mut self,
        view_key: super::NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {
        let view = self
            .views
            .get_mut(view_key)
            .expect("set_view_bounds requires a valid key");

        if position.is_none() && size.is_none() {
            return;
        }

        view.set_bounds(position, size);

        self.recompute_bounds();
    }

    fn restack_view(&mut self, view_key: super::NativeViewKey, stack_order: super::StackOrder) {
        let view = self
            .views
            .get(view_key)
            .expect("set_view_bounds requires a valid key");
        match stack_order {
            super::StackOrder::StackAbove(ref_view_key) => {
                let ref_view = self
                    .views
                    .get(ref_view_key)
                    .expect("set_view_bounds requires a valid key");
                self.display_state.request_void(|conn| {
                    let aux = ConfigureWindowAux::new()
                        .sibling(ref_view.window)
                        .stack_mode(xproto::StackMode::ABOVE);
                    conn.configure_window(view.window, &aux)
                });
            }
            super::StackOrder::StackBelow(ref_view_key) => {
                let ref_view = self
                    .views
                    .get(ref_view_key)
                    .expect("set_view_bounds requires a valid key");
                self.display_state.request_void(|conn| {
                    let aux = ConfigureWindowAux::new()
                        .sibling(ref_view.window)
                        .stack_mode(xproto::StackMode::BELOW);
                    conn.configure_window(view.window, &aux)
                });
            }
        }
    }

    fn total_bounds(&self) -> dpi::PhysicalSize<u32> {
        self.current_bounds
    }

    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        self.display_state.request_void(|conn| {
            let aux = ConfigureWindowAux::new().x(position.x).y(position.y);
            conn.configure_window(self.window, &aux)
        });
    }

    fn set_mapped(&mut self, mapped: bool) {
        self.display_state.request_void(|conn| {
            if mapped {
                conn.map_window(self.window)
            } else {
                conn.unmap_window(self.window)
            }
        });
    }
}

impl XcbView {
    fn set_bounds(
        &mut self,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {
        position.inspect(|pos| self.position = *pos);
        size.inspect(|size| self.size = *size);

        self.display_state.request_void(|conn| {
            let mut aux = ConfigureWindowAux::new();
            if let Some(position) = position {
                aux = aux.x(position.x).y(position.y);
            }
            if let Some(size) = size {
                aux = aux.width(size.width).height(size.height);
            }

            conn.configure_window(self.window, &aux)
        });
    }
}
impl Drop for XcbView {
    fn drop(&mut self) {
        self.display_state
            .request_void(|conn| conn.destroy_window(self.window));
    }
}

impl NativeView for XcbViewHandle {
    fn key(&self) -> NativeViewKey {
        self.view_key
    }

    fn display_handle_src(&self) -> &dyn HasDisplayHandle {
        self
    }

    fn window_handle_src(&self) -> &dyn HasWindowHandle {
        self
    }

    fn gl_api_preference(&self) -> DisplayApiPreference {
        DisplayApiPreference::Egl
    }
}
impl HasDisplayHandle for XcbViewHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let xcb_conn = self.display_state.conn.get_raw_xcb_connection();
        let screen = self.display_state.screen;

        let raw_handle = XcbDisplayHandle::new(NonNull::new(xcb_conn), screen);
        unsafe { Ok(DisplayHandle::borrow_raw(raw_handle.into())) }
    }
}
impl HasWindowHandle for XcbViewHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let mut raw_handle = XcbWindowHandle::new(NonZero::new(self.window).unwrap());
        raw_handle.visual_id = NonZero::new(self.visual_id);

        unsafe { Ok(WindowHandle::borrow_raw(raw_handle.into())) }
    }
}
