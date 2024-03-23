use std::{
    ffi::{c_char, c_int, c_void, CStr},
    sync::{Arc, RwLock},
};

use crate::error::M64PError;

use ash::vk;
use m64prs_sys::{GLAttribute, RenderMode, Size2D, VideoFlags, VideoMode};

#[derive(Clone, PartialEq, Eq)]
pub struct APIVersion {
    pub api_type: m64prs_sys::PluginType,
    pub plugin_version: c_int,
    pub api_version: c_int,
    pub plugin_name: &'static str,
    pub capabilities: c_int,
}

/// Result type for callbacks into Mupen64Plus.
pub type FFIResult<T> = Result<T, M64PError>;

/// Trait for implementing the video extension. The function APIs have been Rustified for convenience.
/// The functions in this trait are unsafe, as there are some thread-safety guarantees that need to be upheld from Mupen's side.
pub trait VideoExtension {
    /// Initializes the video extension with the specified graphics API.
    unsafe fn init_with_render_mode(&mut self, mode: RenderMode) -> FFIResult<()>;
    /// Shuts down the video extension.
    unsafe fn quit(&mut self) -> FFIResult<()>;

    /// Lists the available resolutions when rendering in full screen.
    unsafe fn list_fullscreen_modes(&mut self) -> FFIResult<impl AsRef<[Size2D]>>;
    /// Lists the available refresh rates for a specific fullscreen resolution.
    unsafe fn list_fullscreen_rates(&mut self, size: Size2D) -> FFIResult<impl AsRef<[c_int]>>;

    /// Sets up a render context with the specified dimensions and current OpenGL attributes.
    unsafe fn set_video_mode(
        &mut self,
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;
    /// Sets up a render context with the specified dimensions, refresh rate, and current OpenGL attributes.
    unsafe fn set_video_mode_with_rate(
        &mut self,
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;

    /// Sets the window title.
    unsafe fn set_caption(&mut self, title: &CStr) -> FFIResult<()>;
    /// Toggles fullscreen.
    unsafe fn toggle_full_screen(&mut self) -> FFIResult<()>;
    /// Resizes the render context to the specified width and height.
    unsafe fn resize_window(&mut self, width: c_int, height: c_int) -> FFIResult<()>;

    /// Grabs an OpenGL function with the specified name.
    unsafe fn gl_get_proc_address(&mut self, symbol: &CStr) -> *mut c_void;
    /// Sets an OpenGL attribute. This is called before [`VideoExtension::set_video_mode`].
    unsafe fn gl_set_attribute(&mut self, attr: GLAttribute, value: c_int) -> FFIResult<()>;
    /// Gets an OpenGL attribute. This is generally called after [`VideoExtension::set_video_mode`].
    unsafe fn gl_get_attribute(&mut self, attr: GLAttribute) -> FFIResult<c_int>;
    /// Swaps buffers on the current render context.
    unsafe fn gl_swap_buffers(&mut self) -> FFIResult<()>;
    /// Gets the default FBO for this render context.
    unsafe fn gl_get_default_framebuffer(&mut self) -> u32;

    /// Acquires a Vulkan surface from the window.
    unsafe fn vk_get_surface(&mut self, inst: vk::Instance) -> FFIResult<vk::SurfaceKHR>;
    /// Lists the extensions needed to use [`VideoExtension::vk_get_surface`]
    unsafe fn vk_get_instance_extensions(&mut self) -> FFIResult<&'static [*const c_char]>;
}
