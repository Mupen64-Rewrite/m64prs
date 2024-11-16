use std::sync::{Arc, LazyLock};

use as_raw_xcb_connection::AsRawXcbConnection;
use gdk_x11::ffi::{gdk_x11_display_get_xdisplay, GdkX11Display};
use glib::translate::{Stash, ToGlibPtr};
use gtk::prelude::*;
use x11rb::xcb_ffi::XCBConnection;

pub struct DisplayState {
    pub conn: XCBConnection,
    pub screen: u32,
}

impl DisplayState {
}

mod sealed {
    pub trait Sealed {}
}

pub trait X11DisplayExt: sealed::Sealed {
    fn display_state(&self) -> Arc<DisplayState>;
}
pub trait X11SurfaceExt: sealed::Sealed {
    fn xid(&self) -> u32;
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

        let conn = unsafe {
            let ffi_display: Stash<'_, *mut GdkX11Display, _> = self.to_glib_none();
            let xlib_display_ptr = gdk_x11_display_get_xdisplay(ffi_display.0);
            let xcb_display_ptr =
                tiny_xlib::Display::from_ptr(xlib_display_ptr).as_raw_xcb_connection();

            XCBConnection::from_raw_xcb_connection(xcb_display_ptr as *mut _, false)
                .expect("failed to reuse XCB connection")
        };

        let screen = self.screen().screen_number() as u32;

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
