use std::{
    ffi::{c_char, c_int, c_void, CStr},
    fmt::Display,
    marker::PhantomData,
    path::Path,
    ptr::{null, null_mut},
};

use dlopen2::wrapper::Container;
use m64prs_sys::api::BasePluginApi;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

use crate::error::{M64PError, PluginLoadError, WrongPluginType};

use super::{core_fn, debug_callback, Core};

/// Extension points for the core.
impl Core {
    /// Attaches a set of plugins to the core. Note that the plugins will *not* be returned
    /// in case of failure, as they will be partially initialized and cannot be reset.
    ///
    /// # Errors
    /// This function can error if:
    /// - Starting up any of the plugins fails
    /// - Attaching any of the plugins fails
    /// - A ROM is not open (yes, this may seem stupid, but it is what it is)
    ///
    /// # Panics
    /// This function will panic if there are plugins are already attached.
    pub fn attach_plugins(&mut self, mut plugins: PluginSet) -> Result<(), PluginLoadError> {
        if self.plugins.is_some() {
            panic!("Plugins have already been attached")
        }

        // startup the four plugins
        let core_ptr = unsafe { self.api.into_raw() };
        plugins
            .graphics
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;
        plugins
            .audio
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;
        plugins
            .input
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;
        plugins
            .rsp
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;

        // This keeps track of the last plugin we attached.
        // 0 - Graphics
        // 1 - Input
        // 2 - Audio
        // 3 - RSP
        let mut init_state: u8 = 0;

        // Monad side-effect abuse. We chain the initializations with and_then
        // since each one requires the previous to finish. The final map_err
        // catches any error that occurred during the pipeline and detaches
        // any plugins that were already attached.
        // SAFETY: the library handles passed to C live only as long as the core
        // does, and will be safe to close after shutdown.
        core_fn(unsafe {
            self.api.base.attach_plugin(
                m64prs_sys::PluginType::Graphics,
                plugins.graphics.api.into_raw() as *mut _,
            )
        })
        .and_then(|_| {
            init_state = 1;
            core_fn(unsafe {
                self.api.base.attach_plugin(
                    m64prs_sys::PluginType::Audio,
                    plugins.audio.api.into_raw() as *mut _,
                )
            })
        })
        .and_then(|_| {
            init_state = 2;
            core_fn(unsafe {
                self.api.base.attach_plugin(
                    m64prs_sys::PluginType::Input,
                    plugins.input.api.into_raw() as *mut _,
                )
            })
        })
        .and_then(|_| {
            init_state = 3;
            core_fn(unsafe {
                self.api.base.attach_plugin(
                    m64prs_sys::PluginType::Rsp,
                    plugins.rsp.api.into_raw() as *mut _,
                )
            })
        })
        .map_err(|err| {
            // SAFETY: detach_plugin does not use any data and is safe to call at any time.
            if init_state >= 3 {
                unsafe { self.api.base.detach_plugin(m64prs_sys::PluginType::Rsp) };
            }
            if init_state >= 2 {
                unsafe { self.api.base.detach_plugin(m64prs_sys::PluginType::Input) };
            }
            if init_state >= 1 {
                unsafe { self.api.base.detach_plugin(m64prs_sys::PluginType::Audio) };
            }
            unsafe {
                self.api
                    .base
                    .detach_plugin(m64prs_sys::PluginType::Graphics)
            };

            PluginLoadError::M64P(err)
        })?;

        self.plugins = Some(plugins);

        Ok(())
    }

    /// Detaches the currently-attached plugins.
    ///
    /// # Panics
    /// This function will panic if there are no plugins attached.
    pub fn detach_plugins(&mut self) {
        if self.plugins.is_none() {
            panic!("Plugins are not attached")
        }

        // SAFETY: detach_plugin does not use any data and is safe to call at any time.
        unsafe {
            self.api
                .base
                .detach_plugin(m64prs_sys::PluginType::Graphics);
            self.api.base.detach_plugin(m64prs_sys::PluginType::Audio);
            self.api.base.detach_plugin(m64prs_sys::PluginType::Input);
            self.api.base.detach_plugin(m64prs_sys::PluginType::Rsp);
        };
        // drop the plugins. this shuts them down.
        self.plugins = None;
    }
}

/// Safely represents the type of a plugin. [`m64prs_sys::PluginType`] offers
/// a null value and a value for the [`Core`][m64prs_sys::PluginType#variant.Core]
/// in addition to the values listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum PluginType {
    Graphics = m64prs_sys::PluginType::Graphics as u32,
    Audio = m64prs_sys::PluginType::Audio as u32,
    Input = m64prs_sys::PluginType::Input as u32,
    Rsp = m64prs_sys::PluginType::Rsp as u32,
}

impl Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Graphics => f.write_str("graphics"),
            PluginType::Audio => f.write_str("audio"),
            PluginType::Input => f.write_str("input"),
            PluginType::Rsp => f.write_str("RSP"),
        }
    }
}

impl From<PluginType> for m64prs_sys::PluginType {
    fn from(value: PluginType) -> Self {
        // safety: every PluginType corresponds to an m64prs_sys::PluginType.
        unsafe { std::mem::transmute(value) }
    }
}

impl TryFrom<m64prs_sys::PluginType> for PluginType {
    type Error = TryFromPrimitiveError<PluginType>;

    fn try_from(value: m64prs_sys::PluginType) -> Result<Self, Self::Error> {
        Self::try_from(value as u32)
    }
}

/// Represents the full set of version data obtainable from Mupen64Plus.
#[derive(Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// The API data exposed.
    pub api_type: PluginType,
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
    /// A bitflag listing capabilities. This is plugin-specific.
    pub capabilities: c_int,
}

/// Holds a loaded instance of a Mupen64Plus plugin.
///
/// The core is responsible for startup/shutdown of plugins; they are never started while you own them.
pub struct Plugin<T: PluginTypeTrait> {
    api: Container<BasePluginApi>,
    _marker: PhantomData<T>,
}

impl<T: PluginTypeTrait> Plugin<T> {
    const TYPE: PluginType = T::TYPE;

    /// Loads a plugin of a specific type.
    ///
    /// # Errors
    /// This function may error if:
    /// - Loading the dynamic library fails
    /// - Obtaining its version info fails
    /// - The plugin's type does not match the type specified.
    ///
    /// If you need to load a plugin of arbitrary type, use [`AnyPlugin::load`].
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PluginLoadError> {
        // SAFETY: we assume the dynamic library loaded here is a plugin. We have
        // no way to tell whether this is malicious, but unfortunately this is by
        // the nature of Mupen64Plus's plugin system.
        let api: Container<BasePluginApi> = unsafe { Container::load(path.as_ref()) }
            .map_err(|err| PluginLoadError::Library(err))?;

        let plugin_type = unsafe {
            let mut value = m64prs_sys::PluginType::Null;
            // SAFETY: this function should only use the borrowed value; it
            // shouldn't store any references.
            core_fn(api.get_version(&mut value, null_mut(), null_mut(), null_mut(), null_mut()))
                .map_err(|err| PluginLoadError::M64P(err))?;
            value
        };
        if plugin_type != T::TYPE.into() {
            return Err(PluginLoadError::InvalidType(plugin_type));
        }

        Ok(Self {
            api,
            _marker: PhantomData,
        })
    }

    /// Obtains version information about this plugin.
    pub fn version_info(&self) -> Result<PluginInfo, M64PError> {
        unsafe {
            let mut plugin_type: m64prs_sys::PluginType = m64prs_sys::PluginType::Null;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();
            let mut capabilites: c_int = 0;

            // SAFETY: this function should only use the borrowed value; it
            // shouldn't store any references.
            core_fn(self.api.get_version(
                &mut plugin_type,
                &mut plugin_version,
                &mut api_version,
                &mut plugin_name,
                &mut capabilites,
            ))?;

            Ok(PluginInfo {
                api_type: plugin_type.try_into().unwrap(),
                plugin_version: plugin_version,
                api_version: api_version,
                plugin_name: CStr::from_ptr(plugin_name),
                capabilities: 0,
            })
        }
    }

    fn startup(&mut self, core_ptr: *mut c_void) -> Result<(), M64PError> {
        let debug_id: &'static CStr = match Self::TYPE {
            PluginType::Rsp => c"m64p(rsp)",
            PluginType::Graphics => c"m64p(gfx)",
            PluginType::Audio => c"m64p(audio)",
            PluginType::Input => c"m64p(input)",
        };

        // SAFETY: We assume the plugin is valid. In addition, the debug ID is a
        // &'static CStr, meaning it will never be freed unexpectedly.
        core_fn(unsafe {
            self.api
                .startup(core_ptr, debug_id.as_ptr() as *mut c_void, debug_callback)
        })
    }
}

impl<T: PluginTypeTrait> Drop for Plugin<T> {
    // SAFETY: The plugin can be shut down at any time, and generally fails
    // fast if it hasn't been started up.
    fn drop(&mut self) {
        unsafe {
            self.api.shutdown();
        }
    }
}

/// Holds an instance of *some* plugin.
#[repr(u32)]
pub enum AnyPlugin {
    Graphics(Plugin<GraphicsPlugin>) = PluginType::Graphics as u32,
    Audio(Plugin<AudioPlugin>) = PluginType::Audio as u32,
    Input(Plugin<InputPlugin>) = PluginType::Input as u32,
    Rsp(Plugin<RspPlugin>) = PluginType::Rsp as u32,
}

impl AnyPlugin {
    /// Loads a plugin of an arbitrary type.
    ///
    /// # Errors
    /// This function may error if:
    /// - Loading the dynamic library fails
    /// - Obtaining its version info fails
    /// - The plugin's type is not a valid plugin type.
    ///
    /// If you need to load a plugin of a specific type, use [`Plugin<T>::load`].
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PluginLoadError> {
        let api: Container<BasePluginApi> = unsafe { Container::load(path.as_ref()) }
            .map_err(|err| PluginLoadError::Library(err))?;

        let plugin_type = unsafe {
            let mut value = m64prs_sys::PluginType::Null;
            core_fn(api.get_version(&mut value, null_mut(), null_mut(), null_mut(), null_mut()))
                .map_err(|err| PluginLoadError::M64P(err))?;
            value
        };

        match plugin_type {
            m64prs_sys::PluginType::Graphics => Ok(Self::Graphics(Plugin {
                api,
                _marker: PhantomData,
            })),
            m64prs_sys::PluginType::Audio => Ok(Self::Audio(Plugin {
                api,
                _marker: PhantomData,
            })),
            m64prs_sys::PluginType::Input => Ok(Self::Input(Plugin {
                api,
                _marker: PhantomData,
            })),
            m64prs_sys::PluginType::Rsp => Ok(Self::Rsp(Plugin {
                api,
                _marker: PhantomData,
            })),
            _ => Err(PluginLoadError::InvalidType(plugin_type)),
        }
    }

    /// Gets this plugin's type.
    pub fn plugin_type(&self) -> PluginType {
        match self {
            AnyPlugin::Graphics(_) => PluginType::Graphics,
            AnyPlugin::Audio(_) => PluginType::Audio,
            AnyPlugin::Input(_) => PluginType::Input,
            AnyPlugin::Rsp(_) => PluginType::Rsp,
        }
    }

    /// Obtains version information about this plugin.
    /// This behaves exactly like [`Plugin<T>::version_info`].
    pub fn version_info(&self) -> Result<PluginInfo, M64PError> {
        match self {
            AnyPlugin::Graphics(plugin) => plugin.version_info(),
            AnyPlugin::Audio(plugin) => plugin.version_info(),
            AnyPlugin::Input(plugin) => plugin.version_info(),
            AnyPlugin::Rsp(plugin) => plugin.version_info(),
        }
    }
}

impl From<Plugin<GraphicsPlugin>> for AnyPlugin {
    fn from(value: Plugin<GraphicsPlugin>) -> Self {
        Self::Graphics(value)
    }
}
impl From<Plugin<AudioPlugin>> for AnyPlugin {
    fn from(value: Plugin<AudioPlugin>) -> Self {
        Self::Audio(value)
    }
}
impl From<Plugin<InputPlugin>> for AnyPlugin {
    fn from(value: Plugin<InputPlugin>) -> Self {
        Self::Input(value)
    }
}
impl From<Plugin<RspPlugin>> for AnyPlugin {
    fn from(value: Plugin<RspPlugin>) -> Self {
        Self::Rsp(value)
    }
}

impl TryFrom<AnyPlugin> for Plugin<GraphicsPlugin> {
    type Error = WrongPluginType;

    fn try_from(value: AnyPlugin) -> Result<Self, Self::Error> {
        match value {
            AnyPlugin::Graphics(plugin) => Ok(plugin),
            other => Err(WrongPluginType::new(
                PluginType::Graphics,
                other.plugin_type(),
            )),
        }
    }
}
impl TryFrom<AnyPlugin> for Plugin<AudioPlugin> {
    type Error = WrongPluginType;

    fn try_from(value: AnyPlugin) -> Result<Self, Self::Error> {
        match value {
            AnyPlugin::Audio(plugin) => Ok(plugin),
            other => Err(WrongPluginType::new(PluginType::Audio, other.plugin_type())),
        }
    }
}
impl TryFrom<AnyPlugin> for Plugin<InputPlugin> {
    type Error = WrongPluginType;

    fn try_from(value: AnyPlugin) -> Result<Self, Self::Error> {
        match value {
            AnyPlugin::Input(plugin) => Ok(plugin),
            other => Err(WrongPluginType::new(PluginType::Input, other.plugin_type())),
        }
    }
}
impl TryFrom<AnyPlugin> for Plugin<RspPlugin> {
    type Error = WrongPluginType;

    fn try_from(value: AnyPlugin) -> Result<Self, Self::Error> {
        match value {
            AnyPlugin::Rsp(plugin) => Ok(plugin),
            other => Err(WrongPluginType::new(PluginType::Rsp, other.plugin_type())),
        }
    }
}

/// A set of plugins that can be loaded into the core.
pub struct PluginSet {
    pub graphics: Plugin<GraphicsPlugin>,
    pub audio: Plugin<AudioPlugin>,
    pub input: Plugin<InputPlugin>,
    pub rsp: Plugin<RspPlugin>,
}

mod sealed {
    pub trait Sealed {}
}

/// Sealed trait implemented by all plugin marker types.
pub trait PluginTypeTrait: sealed::Sealed {
    /// The [`PluginType`] corresponding to this marker type.
    const TYPE: PluginType;
}

/// Marker type for graphics plugins.
pub struct GraphicsPlugin;
impl sealed::Sealed for GraphicsPlugin {}
impl PluginTypeTrait for GraphicsPlugin {
    const TYPE: PluginType = PluginType::Graphics;
}

/// Marker type for audio plugins.
pub struct AudioPlugin;
impl sealed::Sealed for AudioPlugin {}
impl PluginTypeTrait for AudioPlugin {
    const TYPE: PluginType = PluginType::Audio;
}

/// Marker type for input plugins.
pub struct InputPlugin;
impl sealed::Sealed for InputPlugin {}
impl PluginTypeTrait for InputPlugin {
    const TYPE: PluginType = PluginType::Input;
}

/// Marker type for RSP plugins.
pub struct RspPlugin;
impl sealed::Sealed for RspPlugin {}
impl PluginTypeTrait for RspPlugin {
    const TYPE: PluginType = PluginType::Rsp;
}
