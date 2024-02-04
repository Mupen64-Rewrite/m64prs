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

/// Result type for callbacks into Mupen64Plus.
pub type FFIResult<T> = Result<T, M64PError>;

/// Trait for implementing the video extension. This does not expose the underlying C callbacks, but provides a Rusty 
/// API surface that is cleaner, and notably, safer.
pub trait VideoExtension {
    /// Initializes the video extension.
    fn init() -> FFIResult<()>;
    /// Shuts down the video extension.
    fn quit() -> FFIResult<()>;

    /// Lists the available resolutions when rendering in full screen.
    fn list_fullscreen_modes() -> FFIResult<impl Iterator<Item = Size2D>>;
    /// Lists the available refresh rates for a specific fullscreen resolution.
    fn list_fullscreen_rates(size: Size2D) -> FFIResult<impl Iterator<Item = c_int>>;

    /// Sets up a render context with the specified dimensions and current OpenGL attributes.
    fn set_video_mode(
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;
    /// Sets up a render context with the specified dimensions, refresh rate, and current OpenGL attributes.
    fn set_video_mode_with_rate(
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;

    /// Sets the window title.
    fn set_caption(title: &CStr) -> FFIResult<()>;
    /// Toggles fullscreen.
    fn toggle_full_screen() -> FFIResult<()>;
    /// Resizes the render context to the specified width and height.
    fn resize_window(width: c_int, height: c_int) -> FFIResult<()>;

    /// Grabs an OpenGL function with the specified name.
    fn gl_get_proc_address(symbol: &CStr) -> *mut c_void;
    /// Sets an OpenGL attribute. This is called before [`VideoExtension::set_video_mode`].
    fn gl_set_attribute(attr: GLAttribute, value: c_int) -> FFIResult<()>;
    /// Gets an OpenGL attribute. This is generally called after [`VideoExtension::set_video_mode`].
    fn gl_get_attribute(attr: GLAttribute) -> FFIResult<c_int>;
    /// Swaps buffers on the current render context.
    fn gl_swap_buffers() -> FFIResult<()>;
    /// Gets the default FBO for this render context.
    fn gl_get_default_framebuffer() -> u32;
}
