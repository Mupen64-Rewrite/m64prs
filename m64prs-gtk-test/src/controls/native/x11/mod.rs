use std::{ffi::c_int, num::NonZero, ptr::NonNull, sync::Arc};

use gdk::prelude::*;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle, XcbDisplayHandle,
    XcbWindowHandle,
};
use state::{DisplayState, ScopeGuard, X11DisplayExt};
use x11rb::{
    connection::Connection,
    protocol::{
        xfixes::ConnectionExt as XFixesConnectionExt,
        xproto::{
            self, ColormapAlloc, ConfigureWindowAux, ConnectionExt, CreateWindowAux, EventMask,
            VisualClass, Visualtype, WindowClass,
        },
    },
    reexports::x11rb_protocol::protocol::shape,
};

use super::{PlatformSubsurface, SubsurfaceAttributes};

mod state;

pub(super) struct X11Subsurface {
    display_state: Arc<DisplayState>,
    window: u32,
    visual_id: u32,
}

impl X11Subsurface {
    pub(super) fn new(gdk_surface: &gdk_x11::X11Surface, attrs: SubsurfaceAttributes) -> Self {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_x11::X11Display>()
            .unwrap();
        let st = gdk_display.display_state();

        let screen_info = &st.conn.setup().roots[st.screen as usize];

        let parent_window: u32 = gdk_surface
            .xid()
            .try_into()
            .expect("XID should fit into u32");

        let root_window: u32 = gdk_display
            .xrootwindow()
            .try_into()
            .expect("XID should fit into u32");


        let (visual_id, depth, colormap) = match attrs.transparent {
            true => {
                let mut visual_iter = screen_info.allowed_depths.iter().flat_map(|depth| {
                    depth
                        .visuals
                        .iter()
                        .map(move |visual| (visual, depth.depth))
                });
                let (visual_id, depth) = visual_iter.find_map(|(visual, depth)| {
                    (visual.class == xproto::VisualClass::TRUE_COLOR && depth == 32)
                        .then_some((visual.visual_id, depth))
                })
                .expect("transparency support should be available");
                let colormap = {
                    let id = st
                        .conn
                        .generate_id()
                        .expect("new XID generations should succeed");
        
                    st.conn
                        .create_colormap(ColormapAlloc::NONE, id, root_window, visual_id)
                        .expect("colormap creation should succeed");
        
                    id
                };
                (visual_id, depth, Some(colormap))

            },
            false => (screen_info.root_visual, screen_info.root_depth, None),
        };

        let window: u32 = {
            let id = st
                .conn
                .generate_id()
                .expect("new XID generations should succeed");

            let mut create_aux = CreateWindowAux::new()
                .event_mask(EventMask::EXPOSURE);

            if let Some(colormap) = colormap {
                create_aux = create_aux.colormap(colormap);
            }

            st.conn
                .create_window(
                    // depth, id, parent
                    depth,
                    id,
                    parent_window,
                    // x, y, width, height
                    0,
                    0,
                    1,
                    1,
                    // border_width
                    0,
                    // class, visual_id, aux
                    WindowClass::INPUT_OUTPUT,
                    visual_id,
                    &create_aux,
                )
                .and_then(|_| {
                    let configure_aux = ConfigureWindowAux::new()
                        .x(attrs.position.x)
                        .y(attrs.position.y)
                        .width(attrs.surface_size.width)
                        .height(attrs.surface_size.height);

                    st.conn.configure_window(id, &configure_aux)
                })
                .expect("window creation should succeed");
            log::info!("Created X11 window");
            id
        };

        // Set the input region to be empty so we can click through the window.
        {
            let region_id = st
                .conn
                .generate_id()
                .and_then(|id| {
                    st.conn.xfixes_create_region(id, &[])?;
                    Ok(id)
                })
                .expect("new XID generations should succeed");

            let _scope = ScopeGuard::new(|| {
                let _ = st.conn.xfixes_destroy_region(region_id);
            });
            log::debug!("Created XFixes region");

            st.conn
                .xfixes_set_window_shape_region(window, shape::SK::INPUT, 0, 0, region_id)
                .expect("setting region failed");

            log::debug!("Set XFixes region");
        }

        // map the window
        st.conn
            .map_window(window)
            .expect("window map should succeed");

        Self {
            display_state: st,
            window,
            visual_id,
        }
    }
}

impl Drop for X11Subsurface {
    fn drop(&mut self) {
        let st = &self.display_state;
        let _ = st.conn.destroy_window(self.window);
    }
}

impl PlatformSubsurface for X11Subsurface {
    fn set_position(&self, position: dpi::PhysicalPosition<i32>) {
        let st = &self.display_state;
        let cw_aux = ConfigureWindowAux::new().x(position.x).y(position.y);

        st.conn
            .configure_window(self.window, &cw_aux)
            .expect("window configure should succeed");
    }

    fn set_size(&self, size: dpi::PhysicalSize<u32>) {
        let st = &self.display_state;
        let cw_aux = ConfigureWindowAux::new()
            .width(size.width)
            .height(size.height);

        st.conn
            .configure_window(self.window, &cw_aux)
            .expect("window configure should succeed");
    }

    fn display_handle_src(&self) -> &dyn raw_window_handle::HasDisplayHandle {
        self
    }

    fn window_handle_src(&self) -> &dyn raw_window_handle::HasWindowHandle {
        self
    }
}

impl HasDisplayHandle for X11Subsurface {
    fn display_handle<'a>(&'a self) -> Result<DisplayHandle<'a>, HandleError> {
        let ptr = self.display_state.conn.get_raw_xcb_connection();
        let screen = self.display_state.screen;

        let raw_handle = XcbDisplayHandle::new(NonNull::new(ptr), screen as c_int);

        unsafe { Ok(DisplayHandle::borrow_raw(raw_handle.into())) }
    }
}

impl HasWindowHandle for X11Subsurface {
    fn window_handle<'a>(&'a self) -> Result<WindowHandle<'a>, HandleError> {
        let mut raw_handle = XcbWindowHandle::new(
            NonZero::new(self.window).unwrap()
        );
        raw_handle.visual_id = NonZero::new(self.visual_id);

        unsafe { Ok(WindowHandle::borrow_raw(raw_handle.into())) }
    }
}
