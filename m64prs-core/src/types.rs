use std::ffi::{c_int, c_void, CStr};

use crate::{
    ctypes::{self, GLAttribute, Size2D, VideoFlags, VideoMode},
    error::M64PError,
};

#[derive(Clone, PartialEq, Eq)]
pub struct APIVersion {
    pub api_type: ctypes::PluginType,
    pub plugin_version: c_int,
    pub api_version: c_int,
    pub plugin_name: &'static str,
    pub capabilities: c_int,
}

pub type FFIResult<T> = Result<T, M64PError>;

pub trait VideoExtension {
    fn init() -> FFIResult<()>;
    fn quit() -> FFIResult<()>;

    fn list_fullscreen_modes() -> FFIResult<impl Iterator<Item = Size2D>>;
    fn list_fullscreen_rates(size: Size2D) -> FFIResult<impl Iterator<Item = c_int>>;

    fn set_video_mode(
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;
    fn set_video_mode_with_rate(
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;

    fn set_caption(title: &CStr) -> FFIResult<()>;
    fn toggle_full_screen() -> FFIResult<()>;
    fn resize_window(width: c_int, height: c_int) -> FFIResult<()>;

    fn gl_get_proc_address(symbol: &CStr) -> *mut c_void;
    fn gl_set_attribute(attr: GLAttribute, value: c_int) -> FFIResult<()>;
    fn gl_get_attribute(attr: GLAttribute) -> FFIResult<c_int>;
    fn gl_swap_buffers() -> FFIResult<()>;
    fn gl_get_default_framebuffer() -> u32;
}
