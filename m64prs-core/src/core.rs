use std::{
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    fs,
    path::Path,
    ptr::{null, null_mut},
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
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

macro_rules! try_core_api {
    ($e:expr) => {
        match ($e) {
            crate::ctypes::Error::SUCCESS => (),
            err => Err(err.into()),
        }
    };
}

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

struct CoreInner {
    pub api: Container<api::FullCoreApi>,

    pub rsp_plugin: Option<Plugin>,
    pub video_plugin: Option<Plugin>,
    pub audio_plugin: Option<Plugin>,
    pub input_plugin: Option<Plugin>,
}

impl Drop for CoreInner {
    fn drop(&mut self) {
        // TODO: warn on error during this function
        let _ = core_fn(unsafe { self.api.core.detach_plugin(PluginType::GFX) });
        let _ = core_fn(unsafe { self.api.core.detach_plugin(PluginType::AUDIO) });
        let _ = core_fn(unsafe { self.api.core.detach_plugin(PluginType::INPUT) });
        let _ = core_fn(unsafe { self.api.core.detach_plugin(PluginType::RSP) });
        let _ = core_fn(unsafe { self.api.core.shutdown() });
    }
}

/// A loaded instance of Mupen64Plus.
/// 
/// All calls to the core are internally synchronized to keep this API thread-safe.
pub struct Core(RwLock<CoreInner>);

static CORE_INSTANCE: OnceLock<Core> = OnceLock::new();

impl Core {
    /// Initializes Mupen64Plus if it isn't already initialized, or returns the core
    /// if it is already initialized.
    pub fn init(path: impl AsRef<Path>) -> &'static Core {
        CORE_INSTANCE.get_or_init(|| {
            let inner = CoreInner {
                api: unsafe { Container::load(path.as_ref()).unwrap() },
                rsp_plugin: None,
                video_plugin: None,
                audio_plugin: None,
                input_plugin: None,
            };

            core_fn(unsafe {
                inner.api.core.startup(
                    0x02_01_00,
                    null(),
                    null(),
                    null_mut(),
                    debug_callback,
                    null_mut(),
                    state_callback,
                )
            }).unwrap();

            RwLock::new(inner);
        })
    }

    /// Gets the loadded Mupen64Plus instance.
    /// 
    /// # Panics
    /// Panics if the core is not initialized.
    pub fn get() -> &'static Core {
        CORE_INSTANCE.get().expect("Call Core::init() to initialize the core first!")
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
