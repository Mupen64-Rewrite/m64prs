use std::{
    mem,
    ptr::null_mut,
    sync::{Arc, LazyLock},
};

use glib::object::ObjectExt;
use m64prs_gtk_utils::quark;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        UI::WindowsAndMessaging::{
            DefWindowProcW, LoadCursorW, RegisterClassExW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW,
            IDC_ARROW, WNDCLASSEXW,
        },
    },
};

use windows::Win32::System::LibraryLoader::GetModuleHandleA;
pub struct DisplayState {
    pub hinstance: HINSTANCE,
}

mod sealed {
    pub trait Sealed {}
}

pub trait Win32DisplayExt: sealed::Sealed {
    fn display_state(&self) -> Arc<DisplayState>;
}

pub const COMP_WINDOW_CLASS: PCWSTR = w!("m64prs_compositor");

impl sealed::Sealed for gdk_win32::Win32Display {}
impl Win32DisplayExt for gdk_win32::Win32Display {
    fn display_state(&self) -> Arc<DisplayState> {
        static M64PRS_WIN32_DISPLAY_STATE: LazyLock<glib::Quark> =
            quark!("io.github.jgcodes2020.m64prs.win32_display_state");

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

        // Register the compositor window class
        unsafe {
            let comp_window_class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance,
                hCursor: LoadCursorW(HINSTANCE(null_mut()), IDC_ARROW)
                    .expect("default cursor should be loadable"),
                lpszClassName: COMP_WINDOW_CLASS,
                ..Default::default()
            };
            RegisterClassExW(&comp_window_class);
        }

        let state = Arc::new(DisplayState { hinstance });

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
        // WM_NCHITTEST => LRESULT(0),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
