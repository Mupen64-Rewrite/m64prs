use std::sync::{Arc, LazyLock};

use as_raw_xcb_connection::AsRawXcbConnection;
use gdk_x11::ffi::{gdk_x11_display_get_xdisplay, GdkX11Display};
use glib::translate::{Stash, ToGlibPtr};
use gtk::prelude::*;
use m64prs_core::error::M64PError;
use x11rb::{connection::Connection, protocol::xproto, xcb_ffi::XCBConnection};

pub struct DisplayState {
    pub conn: XCBConnection,
    pub screen: i32,
}

mod sealed {
    pub trait Sealed {}
}

pub trait X11DisplayExt: sealed::Sealed {
    fn display_state(&self) -> Result<Arc<DisplayState>, M64PError>;
}

impl sealed::Sealed for gdk_x11::X11Display {}
impl X11DisplayExt for gdk_x11::X11Display {
    fn display_state(&self) -> Result<Arc<DisplayState>, M64PError> {
        static M64PRS_X11_DISPLAY_STATE: LazyLock<glib::Quark> = LazyLock::new(|| {
            const QUARK_NAME: &glib::GStr =
                glib::gstr!("io.github.jgcodes2020.m64prs.x11_display_state");
            glib::Quark::from_static_str(QUARK_NAME)
        });

        // if the display already has globals set, then return then
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            if let Some(p_data) = self.qdata::<Arc<DisplayState>>(*M64PRS_X11_DISPLAY_STATE) {
                return Ok(Arc::clone(p_data.as_ref()));
            }
        }

        // GTK uses Xlib to contact the X server.
        let xlib_display = unsafe {
            let ffi_display: Stash<'_, *mut GdkX11Display, _> = self.to_glib_none();
            let xdisplay_ptr = gdk_x11_display_get_xdisplay(ffi_display.0);

            tiny_xlib::Display::from_ptr(xdisplay_ptr)
        };

        // Xlib is a steaming pile of crap, so we use XCB via x11rb, which is much nicer.
        let conn = unsafe {
            let xcb_display_ptr = xlib_display.as_raw_xcb_connection();

            XCBConnection::from_raw_xcb_connection(xcb_display_ptr as *mut _, false)
                .expect("failed to reuse XCB connection")
        };

        // This is needed to retrieve visual settings on startup, and because OpenGL capabilities
        // may differ per-screen.
        let screen = self.screen().screen_number() as i32;

        let state = Arc::new(DisplayState { conn, screen });
        // set the state now
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            self.set_qdata(*M64PRS_X11_DISPLAY_STATE, state);
            Ok(Arc::clone(
                self.qdata::<Arc<DisplayState>>(*M64PRS_X11_DISPLAY_STATE)
                    .unwrap()
                    .as_ref(),
            ))
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
