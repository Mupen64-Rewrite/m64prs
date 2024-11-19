use std::{num::NonZero, ptr::NonNull, sync::Arc};

use gdk::prelude::*;
use glutin::display::DisplayApiPreference;
use m64prs_core::error::M64PError;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle, XcbDisplayHandle,
    XcbWindowHandle,
};
use state::{DisplayState, ScopeGuard, X11DisplayExt};
use x11rb::{
    connection::Connection,
    errors::ReplyError,
    protocol::{
        xfixes::ConnectionExt as XFixesConnectionExt,
        xproto::{
            self, ConfigureWindowAux, ConnectionExt, CreateWindowAux, EventMask, WindowClass,
        },
    },
    reexports::x11rb_protocol::protocol::shape,
};

use super::{PlatformSubsurface, SubsurfaceAttributes};

mod state;

pub(super) struct XcbSubsurface {
    display_state: Arc<DisplayState>,
    window: u32,
    visual_id: u32,
}

impl XcbSubsurface {
    pub(super) fn new(
        gdk_surface: &gdk_x11::X11Surface,
        attrs: SubsurfaceAttributes,
    ) -> Result<Self, M64PError> {
        macro_rules! check {
            ($exp:expr, $err:expr $(,)?) => {
                match $exp {
                    Ok(value) => value,
                    Err(_) => return Err($err),
                }
            };
            ($exp:expr, $err:expr, $fmt:literal $(, $($args:expr),* $(,)?)?) => {
                match $exp {
                    Ok(value) => value,
                    Err(_) => {
                        ::log::error!($fmt $(, $($args),*)?);
                        return Err($err);
                    },
                }
            };
            ($exp:expr, $err:expr, [$errp:ident] $fmt:literal $(, $($args:expr),* $(,)?)?) => {
                match $exp {
                    Ok(value) => value,
                    Err($errp) => {
                        ::log::error!($fmt $(, $($args),*)?);
                        return Err($err);
                    },
                }
            };
        }

        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_x11::X11Display>()
            .unwrap();
        let st = gdk_display.display_state()?;

        let screen_info = &st.conn.setup().roots[st.screen as usize];

        // X11 encodes window transparency using the framebuffer bit depth.
        // 32-bit color = RGBA -> transparent
        // 24-bit color = RGB  -> opaque
        let depth = match attrs.transparent {
            true => 32u8,
            false => 24u8,
        };

        // X11 encodes a window's pixel format using visuals.
        let visual_id = {
            // Find all visuals for the depth determined above.
            let visuals = check!(
                screen_info
                    .allowed_depths
                    .iter()
                    .find_map(|d| (d.depth == depth).then_some(&d.visuals))
                    .ok_or(()),
                M64PError::SystemFail,
                "X server does not support required depth {}",
                depth
            );
            // Find a "true-color" visual: this type simply yeets pixel data onto the screen.
            check!(
                visuals
                    .iter()
                    .find(|visual| visual.class == xproto::VisualClass::TRUE_COLOR)
                    .ok_or(()),
                M64PError::SystemFail,
                "No visuals available"
            )
            .visual_id
        };

        // Because ancient graphics systems used lookup tables for colour, X11 provides a
        // colormap which maps raw pixel data to actual on-screen colours. However, it
        // doesn't really need to do anything nowadays and really exists for show.
        let colormap = {
            let colormap_id = check!(st.conn.generate_id(), M64PError::SystemFail);

            // Colormaps must correspond with the visual and be hooked onto the same screen
            // as the window we're creating.
            // - ColormapAlloc::NONE: We can't and won't need to change color mappings.
            // - colormap_id: the colormap we're creating.
            // - screen_info.root: the root window will be on the same screen as the window
            //   we're creating. Just use it.
            // - visual_id: the visual we're using for the colormap.
            check!(
                st.conn
                    .create_colormap(
                        xproto::ColormapAlloc::NONE,
                        colormap_id,
                        screen_info.root,
                        visual_id
                    )
                    .map_err(|err| ReplyError::from(err))
                    .and_then(|cookie| cookie.check()),
                M64PError::SystemFail,
                [err] "xcb_create_colormap error: {}", err
            );
            colormap_id
        };

        // We're parenting the window to an existing window, so we need to grab that first.
        let parent_window: u32 = gdk_surface
            .xid()
            .try_into()
            .expect("XID should fit into u32");

        // Now we can finally create the window...
        let window: u32 = {
            let window_id = check!(st.conn.generate_id(), M64PError::SystemFail);

            // Extra settings for create_window that don't fit in its parameter list.
            // - event_mask(EventMask::EXPOSURE): allow the window to respond to being mapped (made visible).
            //   We don't care about input events, or any other events for that matter.
            // - override_redirect(1): ensure the window manager can't mess with this window.
            // - border_pixel(0x00000000u32): This is the colour used by the X server to draw the window border.
            //   Normally, the X server can copy that from the parent window if the depth is the same. However,
            //   we can't guarantee the depths are the same, so we specify it manually.
            // - colormap(colormap): Use the colormap we created earlier.
            let create_aux = CreateWindowAux::new()
                .event_mask(EventMask::EXPOSURE)
                .override_redirect(1)
                .border_pixel(0x00000000u32)
                .colormap(colormap);

            // Create the window.
            // - depth: the bit depth of the window's framebuffer, explained above.
            // - window_id, parent_window: the window we're creating and its parent.
            // - 0, 0, 1, 1: specify a 1x1 window for now. We'll fix it later.
            // - 0: the width of the window border. We don't want one, so we set it to 0.
            // - WindowClass::INPUT_OUTPUT: ensures we can draw graphics to the window.
            // - visual_id: the visual we decided on earlier.
            // - &create_aux: the extra parameters. See above.
            check!(
                st.conn
                    .create_window(
                        depth,
                        window_id,
                        parent_window,
                        0,
                        0,
                        1,
                        1,
                        0,
                        WindowClass::INPUT_OUTPUT,
                        visual_id,
                        &create_aux,
                    )
                    .map_err(|err| ReplyError::from(err))
                    .and_then(|cookie| cookie.check()),
                M64PError::SystemFail,
                [err] "xcb_create_window error: {}", err
            );

            // Settings for configure_window.
            // We're using this call to set this window the position
            // and size we want, hence the "x, y, width, height".
            let configure_aux = ConfigureWindowAux::new()
                .x(attrs.position.x)
                .y(attrs.position.y)
                .width(attrs.surface_size.width)
                .height(attrs.surface_size.height);

            // Move the window to its proper location.
            check!(
                st.conn
                    .configure_window(window_id, &configure_aux)
                    .map_err(|err| ReplyError::from(err))
                    .and_then(|cookie| cookie.check()),
                M64PError::SystemFail,
                [err] "xcb_configure_window error: {}", err
            );
            window_id
        };
        log::info!("Created X11 window");

        // Set the input region to be empty so we can click through the window.
        {
            // Create an empty region.
            let region_id = check!(
                st.conn.generate_id().and_then(|id| {
                    let cookie = st.conn.xfixes_create_region(id, &[])?;
                    cookie.check()?;
                    Ok(id)
                }),
                M64PError::SystemFail,
                [err] "xcb_xfixes_create_region error: {}", err
            );
            // Make sure we get rid of it when leaving this scope.
            let _scope = ScopeGuard::new(|| {
                let _ = st.conn.xfixes_destroy_region(region_id);
            });
            log::debug!("Created XFixes region");

            // Set the region as the region where the window can receive mouse input.
            // Since this is empty, the window is entirely click-through.
            check!(
                st.conn
                    .xfixes_set_window_shape_region(window, shape::SK::INPUT, 0, 0, region_id)
                    .map_err(|err| ReplyError::from(err))
                    .and_then(|cookie| cookie.check()),
                M64PError::SystemFail,
                [err] "xcb_xfixes_set_window_shape_region error: {}", err
            );

            log::debug!("Set XFixes region");
        }

        // Map (i.e. show the window.)
        check!(
            st.conn.map_window(window)
                .map_err(|err| ReplyError::from(err))
                .and_then(|cookie| cookie.check()),
            M64PError::SystemFail,
            [err] "xcb_map_window error: {}", err
        );

        Ok(Self {
            display_state: st,
            window,
            visual_id,
        })
    }
}

impl Drop for XcbSubsurface {
    fn drop(&mut self) {
        let st = &self.display_state;
        let _ = st.conn.destroy_window(self.window);
    }
}

impl PlatformSubsurface for XcbSubsurface {
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

    fn gl_api_preference(&self) -> DisplayApiPreference {
        DisplayApiPreference::Egl
    }
}

impl HasDisplayHandle for XcbSubsurface {
    fn display_handle<'a>(&'a self) -> Result<DisplayHandle<'a>, HandleError> {
        let st = &self.display_state;

        let raw_handle =
            XcbDisplayHandle::new(NonNull::new(st.conn.get_raw_xcb_connection()), st.screen);

        unsafe { Ok(DisplayHandle::borrow_raw(raw_handle.into())) }
    }
}

impl HasWindowHandle for XcbSubsurface {
    fn window_handle<'a>(&'a self) -> Result<WindowHandle<'a>, HandleError> {
        let mut raw_handle = XcbWindowHandle::new(NonZero::new(self.window).unwrap());
        raw_handle.visual_id = NonZero::new(self.visual_id);

        unsafe { Ok(WindowHandle::borrow_raw(raw_handle.into())) }
    }
}
