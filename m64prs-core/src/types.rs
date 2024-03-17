use std::{ffi::{c_char, c_int, c_void, CStr}, sync::{Arc, RwLock}};

use crate::{
    ctypes::{self, GLAttribute, RenderMode, Size2D, VideoFlags, VideoMode},
    error::M64PError,
};

use ash::vk;

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

/// Trait for implementing the video extension. The function APIs have been Rustified for convenience.
/// The functions in this trait are unsafe, as there are some thread-safety guarantees that need to be upheld from Mupen's side.
pub trait VideoExtension {
    /// Initializes the video extension with the specified graphics API.
    unsafe fn init_with_render_mode(mode: RenderMode) -> FFIResult<()>;
    /// Initializes the video extension using OpenGL. This forwards to [`VideoExtension::init_with_render_mode`].
    unsafe fn init() -> FFIResult<()> {
        Self::init_with_render_mode(RenderMode::OPENGL)
    }
    /// Shuts down the video extension.
    unsafe fn quit() -> FFIResult<()>;

    /// Lists the available resolutions when rendering in full screen.
    unsafe fn list_fullscreen_modes() -> FFIResult<impl AsRef<[Size2D]>>;
    /// Lists the available refresh rates for a specific fullscreen resolution.
    unsafe fn list_fullscreen_rates(size: Size2D) -> FFIResult<impl AsRef<[c_int]>>;

    /// Sets up a render context with the specified dimensions and current OpenGL attributes.
    unsafe fn set_video_mode(
        width: c_int,
        height: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;
    /// Sets up a render context with the specified dimensions, refresh rate, and current OpenGL attributes.
    unsafe fn set_video_mode_with_rate(
        width: c_int,
        height: c_int,
        refresh_rate: c_int,
        bits_per_pixel: c_int,
        screen_mode: VideoMode,
        flags: VideoFlags,
    ) -> FFIResult<()>;

    /// Sets the window title.
    unsafe fn set_caption(title: &CStr) -> FFIResult<()>;
    /// Toggles fullscreen.
    unsafe fn toggle_full_screen() -> FFIResult<()>;
    /// Resizes the render context to the specified width and height.
    unsafe fn resize_window(width: c_int, height: c_int) -> FFIResult<()>;

    /// Grabs an OpenGL function with the specified name.
    unsafe fn gl_get_proc_address(symbol: &CStr) -> *mut c_void;
    /// Sets an OpenGL attribute. This is called before [`VideoExtension::set_video_mode`].
    unsafe fn gl_set_attribute(attr: GLAttribute, value: c_int) -> FFIResult<()>;
    /// Gets an OpenGL attribute. This is generally called after [`VideoExtension::set_video_mode`].
    unsafe fn gl_get_attribute(attr: GLAttribute) -> FFIResult<c_int>;
    /// Swaps buffers on the current render context.
    unsafe fn gl_swap_buffers() -> FFIResult<()>;
    /// Gets the default FBO for this render context.
    unsafe fn gl_get_default_framebuffer() -> u32;

    /// Acquires a Vulkan surface from the window.
    unsafe fn vk_get_surface(inst: vk::Instance) -> FFIResult<vk::SurfaceKHR>;
    /// Lists the extensions needed to use [`VideoExtension::vk_get_surface`]
    unsafe fn vk_get_instance_extensions() -> FFIResult<&'static [*const c_char]>;
}
