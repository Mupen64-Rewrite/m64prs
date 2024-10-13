use std::ffi::{c_char, c_float, c_int, c_uint, c_void, CStr, CString};

use crate::error::{M64PError, WrongConfigType};

use ash::vk;
use m64prs_sys::{ConfigType, GLAttribute, RenderMode, Size2D, VideoFlags, VideoMode};

/// Represents the full set of version data obtainable from Mupen64Plus.
#[derive(Clone, PartialEq, Eq)]
pub struct APIVersion {
    /// The API data exposed.
    pub api_type: m64prs_sys::PluginType,
    /// The plugin's current numerical version, represented as a packed bytefield.
    /// Taking the least-significant byte as byte 0:
    /// - byte 2 contains the major version
    /// - byte 1 contains the minor version
    /// - byte 0 contains the patch version
    pub plugin_version: c_int,
    /// The plugin's supported API version represented as a packed bytefield.
    /// Taking the least-significant byte as byte 0:
    /// - byte 2 contains the major version
    /// - byte 1 contains the minor version
    /// - byte 0 contains the patch version
    pub api_version: c_int,
    /// The plugin's name.
    pub plugin_name: &'static CStr,
    /// A bitflag listing capabilities. For the core, available capabilities
    /// are enumerated by [`CoreCaps`][`::m64prs_sys::CoreCaps`].
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

/// Represents the value of a config parameter.
#[derive(Debug, Clone)]
pub enum ConfigValue {
    Int(c_int),
    Float(c_float),
    Bool(bool),
    String(CString)
}

impl ConfigValue {
    /// Returns the equivalent [`ConfigType`] for this value.
    pub fn cfg_type(&self) -> ConfigType {
        match self {
            ConfigValue::Int(_) => ConfigType::Int,
            ConfigValue::Float(_) => ConfigType::Float,
            ConfigValue::Bool(_) => ConfigType::Bool,
            ConfigValue::String(_) => ConfigType::String,
        }
    }

    /// (INTERNAL) Obtains a pointer to this value's data.
    pub(crate) unsafe fn as_ptr(&self) -> *const c_void {
        match self {
            ConfigValue::Int(value) => value as *const c_int as *const c_void,
            ConfigValue::Float(value) => value as *const c_float as *const c_void,
            ConfigValue::Bool(value) => value as *const bool as *const c_void,
            ConfigValue::String(value) => value.as_ptr() as *const c_void,
        }
    }
}

impl From<c_int> for ConfigValue {
    fn from(value: c_int) -> Self {
        Self::Int(value)
    }
}

impl From<c_float> for ConfigValue {
    fn from(value: c_float) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<CString> for ConfigValue {
    fn from(value: CString) -> Self {
        Self::String(value)
    }
}

impl TryInto<c_int> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<c_int, Self::Error> {
        match self {
            ConfigValue::Int(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Int, other.cfg_type()))
        }
    }
}

impl TryInto<c_float> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<c_float, Self::Error> {
        match self {
            ConfigValue::Float(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Float, other.cfg_type()))
        }
    }
}

impl TryInto<bool> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            ConfigValue::Bool(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Bool, other.cfg_type()))
        }
    }
}

impl TryInto<CString> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<CString, Self::Error> {
        match self {
            ConfigValue::String(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::String, other.cfg_type()))
        }
    }
}