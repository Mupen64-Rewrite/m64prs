use std::sync::{Arc, LazyLock};

use as_raw_xcb_connection::AsRawXcbConnection;
use gdk_x11::ffi::{gdk_x11_display_get_xdisplay, GdkX11Display};
use glib::translate::{Stash, ToGlibPtr};
use gtk::prelude::*;
use m64prs_core::error::M64PError;
use x11rb::{
    connection::Connection,
    cookie::VoidCookie,
    errors::{ReplyError, ReplyOrIdError},
    protocol::xproto::{self, ConnectionExt},
    xcb_ffi::XCBConnection,
};

use crate::controls::compositor_view::native::NativeViewAttributes;

pub struct DisplayState {
    pub conn: XCBConnection,
    pub screen: i32,
}

impl DisplayState {
    /// Finds a suitable visual for the provided settings.
    pub(super) fn find_depth_and_visual(&self, transparent: bool) -> (u8, xproto::Visualid) {
        let screen_info = &self.conn.setup().roots[self.screen as usize];

        // X11 encodes window transparency using the framebuffer bit depth.
        // 32-bit color = RGBA -> transparent
        // 24-bit color = RGB  -> opaque
        let depth = match transparent {
            true => 32u8,
            false => 24u8,
        };

        // X11 encodes a window's pixel format using visuals.
        // Find all visuals for the depth determined above.
        let visuals = screen_info
            .allowed_depths
            .iter()
            .find_map(|d| (d.depth == depth).then_some(&d.visuals))
            .expect(&format!("depth {} should be supported", depth));
        // Find a "true-color" visual: this type simply yeets pixel data onto the screen.
        let id = visuals
            .iter()
            .find(|visual| visual.class == xproto::VisualClass::TRUE_COLOR)
            .expect(&format!(
                "there should be true-color visuals available for depth {}",
                depth
            ))
            .visual_id;

        (depth, id)
    }

    pub(super) fn root_window(&self) -> xproto::Window {
        let screen_info = &self.conn.setup().roots[self.screen as usize];
        screen_info.root
    }

    /// Executes a request and checks the cookie.
    pub(super) fn request_void<F>(&self, f: F)
    where
        F: FnOnce(
            &XCBConnection,
        ) -> Result<VoidCookie<'_, XCBConnection>, x11rb::errors::ConnectionError>,
    {
        // do the request
        f(&self.conn)
            .map_err(ReplyError::from)
            // check the cookie
            .and_then(|cookie| cookie.check())
            // unwrap the result
            .expect("checked XCB request should succeed");
    }

    /// Helper function that generates an ID, then executes a request
    /// to initialize it, then checks the cookie.
    pub(super) fn request_with_new_id<F>(&self, f: F) -> u32
    where
        F: FnOnce(
            u32,
            &XCBConnection,
        ) -> Result<VoidCookie<'_, XCBConnection>, x11rb::errors::ConnectionError>,
    {
        self.conn
            // generate the ID
            .generate_id()
            // do the request with the ID
            .and_then(|id| {
                f(id, &self.conn)
                    .map(|cookie| (id, cookie))
                    .map_err(ReplyOrIdError::from)
            })
            // check the request cookie
            .and_then(|(id, cookie)| cookie.check().map(|_| id).map_err(ReplyOrIdError::from))
            // unwrap the resut
            .expect("checked XCB request (with ID) should succeed")
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait X11DisplayExt: sealed::Sealed {
    fn display_state(&self) -> Arc<DisplayState>;
}

impl sealed::Sealed for gdk_x11::X11Display {}
impl X11DisplayExt for gdk_x11::X11Display {
    fn display_state(&self) -> Arc<DisplayState> {
        static M64PRS_X11_DISPLAY_STATE: LazyLock<glib::Quark> = LazyLock::new(|| {
            const QUARK_NAME: &glib::GStr =
                glib::gstr!("io.github.jgcodes2020.m64prs.x11_display_state");
            glib::Quark::from_static_str(QUARK_NAME)
        });

        // if the display already has globals set, then return then
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            if let Some(p_data) = self.qdata::<Arc<DisplayState>>(*M64PRS_X11_DISPLAY_STATE) {
                return Arc::clone(p_data.as_ref());
            }
        }

        // We use XCB to communicate with the X server. This makes our lives
        // pretty easy.
        let (conn, screen) = XCBConnection::connect(None)
            .expect("XCB connection should succeed");
        let screen = screen as i32;

        debug_assert!(screen == self.screen().screen_number());

        let state = Arc::new(DisplayState { conn, screen });
        // set the state now
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            self.set_qdata(*M64PRS_X11_DISPLAY_STATE, state);
            Arc::clone(
                self.qdata::<Arc<DisplayState>>(*M64PRS_X11_DISPLAY_STATE)
                    .unwrap()
                    .as_ref(),
            )
        }
    }
}

pub struct ScopeGuard<F: FnOnce()>(Option<F>);
impl<F: FnOnce()> ScopeGuard<F> {
    pub fn new(f: F) -> Self {
        Self(Some(f))
    }
}
impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        self.0.take().unwrap()()
    }
}
