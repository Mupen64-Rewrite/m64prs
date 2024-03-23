use m64prs_core::types::{FFIResult, VideoExtension};
use m64prs_sys::{GLAttribute, RenderMode, Size2D, VideoFlags, VideoMode};
use std::{
    ffi::{c_char, c_int, c_void, CStr},
    sync::Mutex,
};

struct VidextState {}

impl VideoExtension for VidextState {
    unsafe fn init_with_render_mode(&mut self, mode: RenderMode) -> FFIResult<()> {
        let _ = mode;
        todo!()
    }

    unsafe fn quit(&mut self) -> FFIResult<()> {
        todo!()
    }

    unsafe fn list_fullscreen_modes(&mut self) -> FFIResult<&'static [Size2D]> {
        todo!()
    }

    unsafe fn list_fullscreen_rates(&mut self, size: Size2D) -> FFIResult<&'static [c_int]> {
        let _ = size;
        todo!()
    }

    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()> {
        let _ = (width, height, bits_per_pixel, screen_mode, flags);
        todo!()
    }

    unsafe fn set_video_mode_with_rate(
        &mut self,
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()> {
        let _ = (
            width,
            height,
            refresh_rate,
            bits_per_pixel,
            screen_mode,
            flags,
        );
        todo!()
    }

    unsafe fn set_caption(&mut self, title: &CStr) -> FFIResult<()> {
        let _ = title;
        todo!()
    }

    unsafe fn toggle_full_screen(&mut self) -> FFIResult<()> {
        todo!()
    }

    unsafe fn resize_window(&mut self, width: c_int, height: c_int) -> FFIResult<()> {
        let _ = (width, height);
        todo!()
    }

    unsafe fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void {
        let _ = symbol;
        todo!()
    }

    unsafe fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> FFIResult<()> {
        let _ = (attr, value);
        todo!()
    }

    unsafe fn gl_get_attribute(&mut self, attr: GLAttribute) -> FFIResult<c_int> {
        let _ = attr;
        todo!()
    }

    unsafe fn gl_swap_buffers(&mut self) -> FFIResult<()> {
        todo!()
    }

    unsafe fn gl_get_default_framebuffer(&mut self) -> u32 {
        todo!()
    }

    unsafe fn vk_get_surface(&mut self, inst: ash::vk::Instance) -> FFIResult<ash::vk::SurfaceKHR> {
        let _ = inst;
        todo!()
    }

    unsafe fn vk_get_instance_extensions(&mut self) -> FFIResult<&'static [*const c_char]> {
        todo!()
    }
}

static VIDEXT_INSTANCE: Mutex<VidextState> = Mutex::new(VidextState {});

m64prs_core::vidext_table!([VIDEXT_INSTANCE] pub VIDEXT_TABLE);
