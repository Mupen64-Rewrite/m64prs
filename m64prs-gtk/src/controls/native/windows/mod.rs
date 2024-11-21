use std::{
    ffi::c_void,
    num::NonZero,
    ptr::null_mut,
    sync::{Arc, Mutex},
};

use gdk::prelude::*;
use glutin::display::DisplayApiPreference;
use m64prs_core::error::M64PError;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, Win32WindowHandle, WindowHandle,
    WindowsDisplayHandle,
};
use state::{DisplayState, Win32DisplayExt};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetWindowLongPtrW, MoveWindow, SetParent,
        SetWindowLongPtrW, ShowWindowAsync, GWL_STYLE, HMENU, SW_SHOWNOACTIVATE, WINDOW_EX_STYLE,
        WINDOW_STYLE, WS_CAPTION, WS_CHILD, WS_CLIPSIBLINGS, WS_DISABLED, WS_OVERLAPPED,
        WS_OVERLAPPEDWINDOW, WS_POPUP, WS_SYSMENU,
    },
};

use super::{PlatformSubsurface, SubsurfaceAttributes};

mod state;

pub(super) struct WindowsSubsurface {
    display_state: Arc<DisplayState>,
    hwnd: HWND,
    position: Mutex<dpi::PhysicalPosition<i32>>,
    size: Mutex<dpi::PhysicalSize<u32>>,
}

impl WindowsSubsurface {
    pub(super) fn new(
        gdk_surface: &gdk_win32::Win32Surface,
        attrs: SubsurfaceAttributes,
    ) -> Result<Self, M64PError> {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_win32::Win32Display>()
            .unwrap();
        let st = gdk_display.display_state()?;

        let parent_hwnd: HWND = HWND(gdk_surface.handle().0 as usize as *mut c_void);
        let position = attrs.position;
        let size = attrs.surface_size;

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                state::SUBSURFACE_WINDOW_CLASS,
                windows::core::w!("output window"),
                WS_CHILD,
                position.x,
                position.y,
                size.width as i32,
                size.height as i32,
                parent_hwnd,
                HMENU(null_mut()),
                st.hinstance,
                None,
            )
            .map_err(|err| {
                log::error!("CreateWindowExW failed: {}", err);
                M64PError::SystemFail
            })?
        };

        unsafe {
            let _ = ShowWindowAsync(hwnd, SW_SHOWNOACTIVATE);
        }

        Ok(Self {
            display_state: st,
            hwnd,
            position: Mutex::new(position),
            size: Mutex::new(size),
        })
    }
}

impl Drop for WindowsSubsurface {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

unsafe impl Send for WindowsSubsurface {}
unsafe impl Sync for WindowsSubsurface {}

impl PlatformSubsurface for WindowsSubsurface {
    fn set_position(&self, position: dpi::PhysicalPosition<i32>) {
        unsafe {
            // lock position and size temporarily
            *(self.position.lock().unwrap()) = position;
            let size = *self.size.lock().unwrap();

            let _ = MoveWindow(
                self.hwnd,
                position.x,
                position.y,
                size.width as i32,
                size.height as i32,
                false,
            );
        }
    }

    fn set_size(&self, size: dpi::PhysicalSize<u32>) {
        unsafe {
            // lock position and size temporarily
            *(self.size.lock().unwrap()) = size;
            let position = *self.position.lock().unwrap();

            let _ = MoveWindow(
                self.hwnd,
                position.x,
                position.y,
                size.width as i32,
                size.height as i32,
                false,
            );
        }
    }

    fn display_handle_src(&self) -> &dyn HasDisplayHandle {
        self
    }

    fn window_handle_src(&self) -> &dyn HasWindowHandle {
        self
    }

    fn gl_api_preference(&self) -> DisplayApiPreference {
        DisplayApiPreference::Wgl(Some(self.window_handle().unwrap().as_raw()))
    }
}

impl HasDisplayHandle for WindowsSubsurface {
    fn display_handle<'a>(&'a self) -> Result<DisplayHandle<'a>, HandleError> {
        let raw_handle = WindowsDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(raw_handle.into()) })
    }
}
impl HasWindowHandle for WindowsSubsurface {
    fn window_handle<'a>(&'a self) -> Result<raw_window_handle::WindowHandle<'a>, HandleError> {
        let st = &self.display_state;

        let mut raw_handle =
            Win32WindowHandle::new(NonZero::new(self.hwnd.0 as usize as isize).unwrap());
        raw_handle.hinstance = NonZero::new(st.hinstance.0 as usize as isize);

        Ok(unsafe { WindowHandle::borrow_raw(raw_handle.into()) })
    }
}
