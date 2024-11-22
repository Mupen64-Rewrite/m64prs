use std::{
    mem,
    ptr::{null, null_mut},
    sync::{Arc, LazyLock},
};

use glib::object::ObjectExt;
use windows::{
    core::{s, w, PCSTR, PCWSTR},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{CreateSolidBrush, HBRUSH},
        UI::WindowsAndMessaging::{
            DefWindowProcW, LoadCursorW, RegisterClassExW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, HICON,
            IDC_ARROW, WM_NCHITTEST, WNDCLASSA, WNDCLASSEXW,
        },
    },
};

use m64prs_core::error::M64PError;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;

// GDI RGB macro
#[inline]
const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

pub struct DisplayState {
    pub hinstance: HINSTANCE,
    pub wndclass_atom: u16,
}

mod sealed {
    pub trait Sealed {}
}

pub trait Win32DisplayExt: sealed::Sealed {
    fn display_state(&self) -> Arc<DisplayState>;
}

pub const SUBSURFACE_WINDOW_CLASS: PCWSTR = w!("m64prs_subsurface");

impl sealed::Sealed for gdk_win32::Win32Display {}
impl Win32DisplayExt for gdk_win32::Win32Display {
    fn display_state(&self) -> Arc<DisplayState> {
        static M64PRS_WIN32_DISPLAY_STATE: LazyLock<glib::Quark> = LazyLock::new(|| {
            const QUARK_NAME: &glib::GStr =
                glib::gstr!("io.github.jgcodes2020.m64prs.win32_display_state");
            glib::Quark::from_static_str(QUARK_NAME)
        });

        // if the display already has globals set, then return then
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            if let Some(p_data) = self.qdata::<Arc<DisplayState>>(*M64PRS_WIN32_DISPLAY_STATE) {
                return Arc::clone(p_data.as_ref());
            }
        }

        let hinstance: HINSTANCE = unsafe { GetModuleHandleA(None) }
            .expect("handle to own module should be available")
            .into();

        let bg_brush: HBRUSH = unsafe { CreateSolidBrush(rgb(0, 0, 0)) };

        let wndclass_data = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: HICON(null_mut()),
            hCursor: unsafe { LoadCursorW(HINSTANCE(null_mut()), IDC_ARROW) }
                .expect("default cursor should be loadable"),
            hbrBackground: bg_brush,
            lpszMenuName: PCWSTR(null()),
            lpszClassName: SUBSURFACE_WINDOW_CLASS,
            hIconSm: HICON(null_mut()),
        };

        let wndclass_atom = unsafe { RegisterClassExW(&wndclass_data) };

        let state = Arc::new(DisplayState {
            hinstance,
            wndclass_atom,
        });

        // set the state now
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            self.set_qdata(*M64PRS_WIN32_DISPLAY_STATE, state);
            Arc::clone(
                self.qdata::<Arc<DisplayState>>(*M64PRS_WIN32_DISPLAY_STATE)
                    .unwrap()
                    .as_ref(),
            )
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => LRESULT(0),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
