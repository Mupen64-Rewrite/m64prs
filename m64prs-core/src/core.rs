use std::{
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    fs,
    path::Path,
    ptr::{null, null_mut},
    sync::{atomic::AtomicBool, Mutex, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use dlopen2::wrapper::Container;
use log::{log, Level};

use crate::{
    ctypes::{self, Command, MsgLevel, PluginType, VideoExtensionFunctions},
    error::CoreError,
    types::APIVersion,
};

use crate::error::Result as CoreResult;

mod api;

#[must_use]
fn core_fn(err: ctypes::Error) -> CoreResult<()> {
    if err == ctypes::Error::SUCCESS {
        Ok(())
    } else {
        Err(CoreError::M64P(err.try_into().unwrap()))
    }
}

#[allow(unused)]
unsafe extern "C" fn debug_callback(context: *mut c_void, level: c_int, message: *const c_char) {
    let log_level = match MsgLevel(level as c_uint) {
        MsgLevel::ERROR => Level::Error,
        MsgLevel::WARNING => Level::Warn,
        MsgLevel::INFO => Level::Info,
        MsgLevel::STATUS => Level::Debug,
        MsgLevel::VERBOSE => Level::Trace,
        _ => panic!("Received invalid message level {}", level),
    };
    log!(log_level, "{}", CStr::from_ptr(message).to_str().unwrap());
}

#[allow(unused)]
extern "C" fn state_callback(context: *mut c_void, param: ctypes::CoreParam, new_value: c_int) {}

static CORE_GUARD: Mutex<bool> = Mutex::new(false);

pub struct Core {
    api: Container<api::FullCoreApi>,

    video_plugin: Option<Plugin>,
    audio_plugin: Option<Plugin>,
    input_plugin: Option<Plugin>,
    rsp_plugin: Option<Plugin>,
}

impl Core {
    /// Loads and starts up the core from a given path.
    /// 
    /// # Panics
    /// Only one active instance of `Core` can exist at any given time. Attempting to create a 
    /// second one will cause a panic.
    /// 
    /// # Errors
    /// This function may error if:
    /// - Library loading fails ([`CoreError::Library`])
    /// - Initialization of Mupen64Plus fails ([`CoreError::M64P`])
    fn init(path: impl AsRef<Path>) -> CoreResult<Self> {
        let guard = CORE_GUARD.get_mut().unwrap();
        if *guard {
            drop(guard);
            panic!("Only one instance of Core may be created");
        }

        let api = unsafe { Container::<api::FullCoreApi>::load(path.as_ref()) }
            .map_err(CoreError::Library)?;

        core_fn(unsafe {
            api.core.startup(0x02_01_00, null(), null(), null_mut(), debug_callback, null_mut(), state_callback)
        }).map_err(CoreError::M64P)?;

        *guard = true;
        Ok(Core {
            api,
            video_plugin: None,
            audio_plugin: None,
            input_plugin: None,
            rsp_plugin: None,
        })
    }
}
impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            self.api.core.shutdown()
        }
        // drop the guard so that another core can be constructed
        {
            let guard = CORE_GUARD.get_mut().unwrap();
            *guard = false;
        }
    }
}

/// Holds a loaded instance of a Mupen64Plus plugin. The core is responsible for startup/shutdown of
/// plugins, so plugins will remain unstarted when you have access to them.
pub struct Plugin {
    api: Container<api::BasePluginApi>,
}

impl Plugin {
    /// Loads a plugin from a path.
    pub fn load<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        let plugin = Plugin {
            api: unsafe { Container::load(path.as_ref()) }
                .map_err(|err| CoreError::Library(err))?,
        };

        Ok(plugin)
    }

    /// Obtains version information about this plugin.
    pub fn get_version(&self) -> CoreResult<APIVersion> {
        unsafe {
            let mut plugin_type: PluginType = PluginType::NULL;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();

            core_fn(unsafe {
                self.api.get_version(
                    &mut plugin_type,
                    &mut plugin_version,
                    &mut api_version,
                    &mut plugin_name,
                    null_mut(),
                )
            })?;

            Ok(APIVersion {
                api_type: plugin_type.try_into().unwrap(),
                plugin_version: plugin_version,
                api_version: api_version,
                plugin_name: CStr::from_ptr(plugin_name).to_str().unwrap(),
                capabilities: 0,
            })
        }
    }

    fn startup(&mut self, core_ptr: *mut c_void) -> CoreResult<()> {
        core_fn(unsafe { self.api.startup(core_ptr, null_mut(), debug_callback) })
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        unsafe {
            self.api.shutdown();
        }
    }
}
