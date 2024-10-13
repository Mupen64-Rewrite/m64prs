use std::{
    ffi::{c_char, c_int, c_uint, c_void, CStr, CString}, fmt::Debug, fs, path::Path, pin::Pin, ptr::{null, null_mut}, sync::{mpsc, Mutex}
};

use async_std::sync::Mutex as AsyncMutex;
use dlopen2::wrapper::Container;
use futures::Future;
use log::{log, Level};

use crate::{
    error::{CoreError, M64PError, SavestateError, StartupError},
    types::{APIVersion, ControllerPort},
};

use m64prs_sys::{
    api::{BasePluginApi, FullCoreApi}, Buttons, Command, CoreParam, InputFilterCallback, MsgLevel, PluginType
};

use self::save::{SavestateWaitManager, SavestateWaiter};

mod save;

/// Internal helper function to convert C results to Rust errors.
#[inline(always)]
fn core_fn(err: m64prs_sys::Error) -> Result<(), M64PError> {
    match err {
        m64prs_sys::Error::Success => Ok(()),
        err => Err(M64PError::try_from(err).unwrap()),
    }
}

#[allow(unused)]
unsafe extern "C" fn debug_callback(context: *mut c_void, level: c_int, message: *const c_char) {
    let log_level = match MsgLevel::try_from(level as u32).unwrap() {
        MsgLevel::Error => Level::Error,
        MsgLevel::Warning => Level::Warn,
        MsgLevel::Info => Level::Info,
        MsgLevel::Status => Level::Debug,
        MsgLevel::Verbose => Level::Trace,
        _ => panic!("Received invalid message level {}", level),
    };
    log!(log_level, "{}", CStr::from_ptr(message).to_str().unwrap());
}
#[allow(unused)]
unsafe extern "C" fn vcr_debug_callback(level: MsgLevel, message: *const c_char) {
    let log_level = match level {
        MsgLevel::Error => Level::Error,
        MsgLevel::Warning => Level::Warn,
        MsgLevel::Info => Level::Info,
        MsgLevel::Status => Level::Debug,
        MsgLevel::Verbose => Level::Trace,
    };
    log!(log_level, "{}", CStr::from_ptr(message).to_str().unwrap());
}

#[allow(unused)]
extern "C" fn state_callback(context: *mut c_void, param: CoreParam, value: c_int) {
    let pinned_state = unsafe { &mut *(context as *mut PinnedCoreState) };

    match param {
        CoreParam::StateSaveComplete | CoreParam::StateLoadComplete => {
            pinned_state.st_wait_mgr.on_state_change(param, value);
        }
        _ => (),
    }
}

struct PinnedCoreState {
    pub st_wait_mgr: SavestateWaitManager,
}

static CORE_GUARD: Mutex<bool> = Mutex::new(false);


pub struct Core {
    api: Container<FullCoreApi>,
    pin_state: Pin<Box<PinnedCoreState>>,
    plugins: Option<[Plugin; 4]>,

    save_sender: mpsc::Sender<SavestateWaiter>,
    save_mutex: AsyncMutex<()>,
}

impl Debug for Core {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Core {}")
    }
}

unsafe impl Sync for Core {}

// initialization, helper functions, and cleanup
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
    pub fn init(path: impl AsRef<Path>) -> Result<Self, StartupError> {
        const CORE_DEBUG_ID: &'static CStr = c"mupen64plus-core";

        let mut guard = CORE_GUARD.lock().unwrap();
        if *guard {
            drop(guard);
            panic!("Only one instance of Core may be created");
        }

        let api =
            unsafe { Container::<FullCoreApi>::load(path.as_ref()) }.map_err(StartupError::Library)?;

        let (save_tx, save_rx) = mpsc::channel();

        let mut core = Self {
            api,
            plugins: None,
            pin_state: Box::pin(PinnedCoreState {
                st_wait_mgr: SavestateWaitManager::new(save_rx),
            }),
            save_sender: save_tx,
            save_mutex: AsyncMutex::new(()),
        };

        unsafe {
            core_fn(core.api.base.startup(
                0x02_01_00,
                null(),
                null(),
                CORE_DEBUG_ID.as_ptr() as *mut c_void,
                debug_callback,
                &mut *core.pin_state as *mut PinnedCoreState as *mut c_void,
                state_callback,
            )).map_err(|err| StartupError::CoreInit(err))?;
        };

        *guard = true;
        Ok(core)
    }

    #[inline(always)]
    unsafe fn do_command_ip(
        &self,
        command: Command,
        int_param: c_int,
        ptr_param: *mut c_void,
    ) -> Result<(), M64PError> {
        core_fn(unsafe { self.api.base.do_command(command, int_param, ptr_param) })
    }

    #[inline(always)]
    unsafe fn do_command_p(&self, command: Command, ptr_param: *mut c_void) -> Result<(), M64PError> {
        self.do_command_ip(command, 0, ptr_param)
    }

    #[inline(always)]
    fn do_command_i(&self, command: Command, int_param: c_int) -> Result<(), M64PError> {
        unsafe { self.do_command_ip(command, int_param, null_mut()) }
    }

    #[inline(always)]
    fn do_command(&self, command: Command) -> Result<(), M64PError> {
        unsafe { self.do_command_ip(command, 0, null_mut()) }
    }
}
impl Drop for Core {
    fn drop(&mut self) {
        // shutdown the core before it's freed
        unsafe { self.api.base.shutdown() };
        // drop the guard so that another core can be constructed
        {
            let mut guard = CORE_GUARD.lock().unwrap();
            *guard = false;
        }
    }
}

// Extension points (plugins and vidext)
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
    ) -> Result<(), CoreError> {
        if self.plugins.is_some() {
            panic!("Plugins have already been attached")
        }
        // check all plugin types
        if !gfx_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Graphics)
        {
            return Err(CoreError::PluginInvalid(PluginType::Graphics));
        }
        if !audio_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Audio)
        {
            return Err(CoreError::PluginInvalid(PluginType::Audio));
        }
        if !input_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Input)
        {
            return Err(CoreError::PluginInvalid(PluginType::Input));
        }
        if !rsp_plugin
            .get_type()
            .is_ok_and(|ptype| ptype == PluginType::Rsp)
        {
            return Err(CoreError::PluginInvalid(PluginType::Rsp));
        }

        // startup the four plugins
        let core_ptr = unsafe { self.api.into_raw() };
        gfx_plugin.startup(core_ptr).map_err(|err| CoreError::M64P(err))?;
        audio_plugin.startup(core_ptr).map_err(|err| CoreError::M64P(err))?;
        input_plugin.startup(core_ptr).map_err(|err| CoreError::M64P(err))?;
        rsp_plugin.startup(core_ptr).map_err(|err| CoreError::M64P(err))?;

        // attach all plugins. If one fails, detach everything.
        if let Err(err) = core_fn(unsafe {
            self.api
                .base
                .attach_plugin(PluginType::Graphics, gfx_plugin.api.into_raw())
        }) {
            unsafe { self.api.base.detach_plugin(PluginType::Graphics) };
            return Err(CoreError::M64P(err));
        }
        if let Err(err) = core_fn(unsafe {
            self.api
                .base
                .attach_plugin(PluginType::Audio, audio_plugin.api.into_raw())
        }) {
            unsafe { self.api.base.detach_plugin(PluginType::Graphics) };
            unsafe { self.api.base.detach_plugin(PluginType::Audio) };
            return Err(CoreError::M64P(err));
        }
        if let Err(err) = core_fn(unsafe {
            self.api
                .base
                .attach_plugin(PluginType::Input, input_plugin.api.into_raw())
        }) {
            unsafe { self.api.base.detach_plugin(PluginType::Graphics) };
            unsafe { self.api.base.detach_plugin(PluginType::Audio) };
            unsafe { self.api.base.detach_plugin(PluginType::Input) };
            return Err(CoreError::M64P(err));
        }
        if let Err(err) = core_fn(unsafe {
            self.api
                .base
                .attach_plugin(PluginType::Rsp, rsp_plugin.api.into_raw())
        }) {
            unsafe { self.api.base.detach_plugin(PluginType::Graphics) };
            unsafe { self.api.base.detach_plugin(PluginType::Audio) };
            unsafe { self.api.base.detach_plugin(PluginType::Input) };
            unsafe { self.api.base.detach_plugin(PluginType::Rsp) };
            return Err(CoreError::M64P(err));
        }

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

    /// Overrides the functions used by graphics plugins to setup a window and OpenGL/Vulkan context.
    ///
    /// The typical way of acquiring a [`m64prs_sys::VideoExtensionFunctions`] is to generate it
    /// via the [`vidext_table!()`][`crate::vidext_table!`] macro and [`VideoExtension`][`crate::types::VideoExtension`] trait.
    pub fn override_vidext(
        &mut self,
        vidext: &m64prs_sys::VideoExtensionFunctions,
    ) -> Result<(), M64PError> {
        // This is actually safe, since Mupen copies the table.
        core_fn(unsafe {
            self.api.base.override_vidext(
                vidext as *const m64prs_sys::VideoExtensionFunctions
                    as *mut m64prs_sys::VideoExtensionFunctions,
            )
        })
    }
}

// Synchronous core commands
impl Core {
    /// Opens a ROM that is pre-loaded into memory.
    pub fn open_rom(&mut self, rom_data: &[u8]) -> Result<(), M64PError> {
        unsafe {
            self.do_command_ip(
                Command::RomOpen,
                rom_data.len() as c_int,
                rom_data.as_ptr() as *mut c_void,
            )
        }
    }

    /// Closes a currently open ROM.
    pub fn close_rom(&mut self) -> Result<(), M64PError> {
        self.do_command(Command::RomClose)
    }

    pub fn set_frame_callback(&mut self, callback: unsafe extern "C" fn(c_uint)) -> Result<(), M64PError> {
        unsafe {
            self.do_command_p(
                Command::SetFrameCallback,
                callback as *const c_void as *mut c_void,
            )
        }
    }

    pub fn clear_frame_callback(&mut self) -> Result<(), M64PError> {
        unsafe { self.do_command_p(Command::SetFrameCallback, null_mut()) }
    }

    /// Executes the currently-open ROM.
    pub fn execute(&self) -> Result<(), M64PError> {
        self.do_command(Command::Execute)
    }
}

// Asynchronous core commands
impl Core {
    /// Stops the currently-running ROM.
    pub fn stop(&self) -> Result<(), M64PError> {
        // TODO: add async waiter that waits on emulator state
        self.do_command(Command::Stop)
    }
    /// Pauses the currently-running ROM.
    pub fn pause(&self) -> Result<(), M64PError> {
        // TODO: add async waiter that waits on emulator state
        self.do_command(Command::Pause)
    }
    pub fn resume(&self) -> Result<(), M64PError> {
        // TODO: add async waiter that waits on emulator state
        self.do_command(Command::Resume)
    }
    pub fn advance_frame(&self) -> Result<(), M64PError> {
        // TODO: add async waiter that waits on emulator state
        self.do_command(Command::AdvanceFrame)
    }

    /// Notifies the graphics plugin of a change in the window's size.
    pub fn notify_resize(&self, width: u16, height: u16) -> Result<(), M64PError> {
        let size_packed = (((width as u32) << 16) | (height as u32)) as c_int;
        unsafe {
            self.do_command_ip(
                Command::CoreStateSet,
                u32::from(CoreParam::VideoSize) as c_int,
                &size_packed as *const c_int as *mut c_void,
            )
        }
    }
}

// Savestates
impl Core {
    /// Saves game state to the current slot.
    pub async fn save_state(&self) -> Result<(), SavestateError> {
        let _lock = self.save_mutex.lock().await;
        let res = self.save_state_inner().await;
        res
    }

    /// Loads game state from the current slot.
    pub async fn load_state(&self) -> Result<(), SavestateError> {
        let _lock = self.save_mutex.lock().await;
        let res = self.load_state_inner().await;
        res
    }

    fn save_state_inner(&self) -> impl Future<Output = Result<(), SavestateError>> {
        // create transmission channel for savestate result
        let (mut future, waiter) = save::save_pair(CoreParam::StateSaveComplete);
        self.save_sender
            .send(waiter)
            .expect("Waiter queue disconnected!");
        // initiate the save operation. This is guaranteed to trip the waiter at some point.
        if let Err(error) =
            core_fn(unsafe { self.api.base.do_command(Command::StateSave, 0, null_mut()) })
        {
            future.fail_early(error);
        }

        future
    }

    fn load_state_inner(&self) -> impl Future<Output = Result<(), SavestateError>> {
        let (mut future, waiter) = save::save_pair(CoreParam::StateLoadComplete);
        self.save_sender
            .send(waiter)
            .expect("Waiter queue disconnected!");

        if let Err(error) =
            core_fn(unsafe { self.api.base.do_command(Command::StateLoad, 0, null_mut()) })
        {
            future.fail_early(error);
        }

        future
    }
}

// CONFIGURATION API
// ===================

// Configuration
impl Core {
    /// Runs the provided callback once per available config section.
    pub fn cfg_for_each_section<F: FnMut(&CStr)>(&self, mut callback: F) -> Result<(), M64PError> {
        unsafe extern "C" fn run_callback<F: FnMut(&CStr)>(context: *mut c_void, name: *const c_char) {
            let function: &mut F = &mut *(context as *mut F);
            function(CStr::from_ptr(name));
        }

        core_fn(unsafe {
            self.api.config.list_sections(&mut callback as *mut F as *mut c_void, run_callback::<F>)
        })?;

        Ok(())
    }
    /// Opens the config section with the given name.
    pub fn cfg_open(&self, name: &CStr) -> Result<ConfigSection, M64PError> {
        let mut handle: m64prs_sys::Handle = null_mut();
        core_fn(unsafe {
            self.api.config.open_section(name.as_ptr(), &mut handle as *mut m64prs_sys::Handle)
        })?;

        Ok(ConfigSection { core: self, name: name.to_owned(), handle: handle })
    }
}
struct ConfigSection<'a> {
    core: &'a Core,
    name: CString,
    handle: m64prs_sys::Handle
}

impl<'a> ConfigSection<'a> {
    pub fn save(&self) -> Result<(), M64PError> {
        core_fn(unsafe {
            self.core.api.config.save_section(self.name.as_ptr())
        })
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
