use std::{ffi::c_void, ptr::null_mut, sync::Arc};

use gdk::prelude::*;
use slotmap::DenseSlotMap;
use state::{DisplayState, Win32DisplayExt};
use windows::Win32::{
    Foundation::HWND, Graphics::{Direct3D::D3D_DRIVER_TYPE_HARDWARE, Direct3D11::{D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION}}, UI::WindowsAndMessaging::{CreateWindowExW, HMENU, WINDOW_EX_STYLE, WS_CHILD}
};

use crate::controls::native::SubsurfaceAttributes;

use super::{NativeCompositor, NativeViewKey};

mod state;
mod dcomp;

struct WindowsCompositor {
    display_state: Arc<DisplayState>,
    views: DenseSlotMap<NativeViewKey, WindowsView>,

    current_bounds: dpi::PhysicalSize<u32>,
    mapped: bool,

    window: HWND,
}

struct WindowsView {}

struct WindowsViewHandle {}

impl WindowsCompositor {
    pub(super) fn new(
        gdk_surface: &gdk_win32::Win32Surface,
        attrs: SubsurfaceAttributes,
    ) -> Self {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_win32::Win32Display>()
            .unwrap();
        let st = gdk_display.display_state();

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
            .expect("crap")
        };

        todo!()
    }
}

impl NativeCompositor for WindowsCompositor {
    fn new_view(&mut self, attrs: super::NativeViewAttributes) -> Box<dyn super::NativeView> {
        todo!()
    }

    fn delete_view(&mut self, view: super::NativeViewKey) {
        todo!()
    }

    fn set_view_bounds(
        &mut self,
        key: super::NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {
        todo!()
    }

    fn restack_view(&mut self, key: super::NativeViewKey, stack_order: super::StackOrder) {
        todo!()
    }

    fn total_bounds(&self) -> dpi::PhysicalSize<u32> {
        todo!()
    }

    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        todo!()
    }

    fn set_mapped(&mut self, mapped: bool) {
        todo!()
    }
}
