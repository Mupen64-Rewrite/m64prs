use std::{
    ffi::{c_char, c_int, c_void, CStr},
    path::Path,
    ptr::{null, null_mut},
};

use dlopen2::wrapper::Container;
use m64prs_sys::{api::BasePluginApi, PluginType};

use crate::{
    error::{M64PError, PluginLoadError},
    types::APIVersion,
};

use super::{core_fn, debug_callback, Core};


/// Extension points for the core.
impl Core {
    /// Attaches the four plugins to the core.
    ///
    /// # Errors
    /// This function can error if:
    /// - The plugins passed in do not match their supposed type (e.g. `gfx_plugin` expects a graphics plugin)
    /// - Starting up any of the four plugins fails
    /// - Attaching any of the four plugins fails
    /// - A ROM is not open (yes, this may seem stupid, but it is what it is)
    pub fn attach_plugins(
        &mut self,
        mut gfx_plugin: Plugin,
        mut audio_plugin: Plugin,
        mut input_plugin: Plugin,
        mut rsp_plugin: Plugin,
    ) -> Result<(), PluginLoadError> {
        if self.plugins.is_some() {
            panic!("Plugins have already been attached")
        }
        // check all plugin types
        if !gfx_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Graphics)
        {
            return Err(PluginLoadError::PluginInvalid(PluginType::Graphics));
        }
        if !audio_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Audio)
        {
            return Err(PluginLoadError::PluginInvalid(PluginType::Audio));
        }
        if !input_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Input)
        {
            return Err(PluginLoadError::PluginInvalid(PluginType::Input));
        }
        if !rsp_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Rsp)
        {
            return Err(PluginLoadError::PluginInvalid(PluginType::Rsp));
        }

        // startup the four plugins
        let core_ptr = unsafe { std::mem::transmute::<_, *mut c_void>(self.api.into_raw()) };
        gfx_plugin
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;
        audio_plugin
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;
        input_plugin
            .startup(core_ptr)
            .map_err(|err| PluginLoadError::M64P(err))?;
        rsp_plugin
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
        core_fn(unsafe {
            self.api
                .base
                .attach_plugin(PluginType::Graphics, gfx_plugin.api.into_raw())
        })
        .and_then(|_| {
            init_state = 1;
            core_fn(unsafe {
                self.api
                    .base
                    .attach_plugin(PluginType::Audio, audio_plugin.api.into_raw())
            })
        })
        .and_then(|_| {
            init_state = 2;
            core_fn(unsafe {
                self.api
                    .base
                    .attach_plugin(PluginType::Input, input_plugin.api.into_raw())
            })
        })
        .and_then(|_| {
            init_state = 3;
            core_fn(unsafe {
                self.api
                    .base
                    .attach_plugin(PluginType::Rsp, rsp_plugin.api.into_raw())
            })
        })
        .map_err(|err| {
            if init_state >= 3 {
                unsafe { self.api.base.detach_plugin(PluginType::Rsp) };
            }
            if init_state >= 2 {
                unsafe { self.api.base.detach_plugin(PluginType::Input) };
            }
            if init_state >= 1 {
                unsafe { self.api.base.detach_plugin(PluginType::Audio) };
            }
            unsafe { self.api.base.detach_plugin(PluginType::Graphics) };

            PluginLoadError::M64P(err)
        })?;

        self.plugins = Some([gfx_plugin, audio_plugin, input_plugin, rsp_plugin]);

        Ok(())
    }

    /// Detaches all plugins.
    pub fn detach_plugins(&mut self) {
        if self.plugins.is_none() {
            panic!("Plugins are not attached")
        }

        // detach plugins from core.
        unsafe { self.api.base.detach_plugin(PluginType::Graphics) };
        unsafe { self.api.base.detach_plugin(PluginType::Audio) };
        unsafe { self.api.base.detach_plugin(PluginType::Input) };
        unsafe { self.api.base.detach_plugin(PluginType::Rsp) };
        // drop the plugins. this shuts them down.
        self.plugins = None;
    }
}

/// Holds a loaded instance of a Mupen64Plus plugin.
///
/// The core is responsible for startup/shutdown of plugins; they are never started while you own them.
pub struct Plugin {
    api: Container<BasePluginApi>,
}

impl Plugin {
    /// Loads a plugin from a path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, dlopen2::Error> {
        let plugin = Plugin {
            api: unsafe { Container::load(path.as_ref()) }?,
        };

        Ok(plugin)
    }

    /// Gets the type of this plugin.
    pub fn get_type(&self) -> Result<PluginType, M64PError> {
        let mut plugin_type: PluginType = PluginType::Null;
        core_fn(unsafe {
            self.api.get_version(
                &mut plugin_type,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        })?;

        Ok(plugin_type)
    }

    /// Obtains version information about this plugin.
    pub fn get_version(&self) -> Result<APIVersion, M64PError> {
        unsafe {
            let mut plugin_type: PluginType = PluginType::Null;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();

            core_fn(self.api.get_version(
                &mut plugin_type,
                &mut plugin_version,
                &mut api_version,
                &mut plugin_name,
                null_mut(),
            ))?;

            Ok(APIVersion {
                api_type: plugin_type.try_into().unwrap(),
                plugin_version: plugin_version,
                api_version: api_version,
                plugin_name: CStr::from_ptr(plugin_name),
                capabilities: 0,
            })
        }
    }

    fn startup(&mut self, core_ptr: *mut c_void) -> Result<(), M64PError> {
        let debug_id: &'static CStr = match self.get_type().unwrap() {
            PluginType::Rsp => c"m64p(rsp)",
            PluginType::Graphics => c"m64p(gfx)",
            PluginType::Audio => c"m64p(audio)",
            PluginType::Input => c"m64p(input)",
            _ => c"m64p(??)"
        };

        core_fn(unsafe { self.api.startup(core_ptr, debug_id.as_ptr() as *mut c_void, debug_callback) })
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        unsafe {
            self.api.shutdown();
        }
    }
}
