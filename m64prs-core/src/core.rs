use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    fmt::Debug,
    path::Path,
    ptr::{null, null_mut},
    sync::{mpsc, Mutex},
};

use async_std::sync::Mutex as AsyncMutex;
use dlopen2::wrapper::Container;
use emu_state::{EmuStateWaitManager, EmuStateWaiter};
use log::{log, Level};
use num_enum::TryFromPrimitive;
use plugin::PluginSet;
use slotmap::HopSlotMap;
use tas_callbacks::ffi::FFIHandler;

use crate::error::{M64PError, StartupError};

use m64prs_sys::{api::FullCoreApi, Command, CoreParam, MsgLevel};

use self::save::{SavestateWaitManager, SavestateWaiter};

pub mod config;
pub mod emu_state;
pub mod plugin;
pub mod save;
pub mod tas_callbacks;
pub mod vidext;

pub use config::ConfigSection;
pub use plugin::Plugin;



slotmap::new_key_type! {
    pub struct StateHandlerKey;
}

/// Trait alias for closures that can handle state changes from Mupen.
pub trait StateHandler: FnMut(CoreParam, c_int) + Send + Sync {}

impl<F> StateHandler for F
where
    F: FnMut(CoreParam, c_int) + Send + Sync {}


pub struct Core {
    pin_state: Box<Mutex<PinnedCoreState>>,

    plugins: Option<PluginSet>,

    st_sender: mpsc::Sender<SavestateWaiter>,
    st_mutex: AsyncMutex<()>,

    emu_sender: mpsc::Sender<EmuStateWaiter>,

    // These handlers represent some arbitrary object that
    // we are holding onto until we don't need it.
    input_handler: Option<Box<dyn FFIHandler>>,
    audio_handler: Option<Box<dyn FFIHandler>>,

    api: Container<FullCoreApi>,
}

impl Debug for Core {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Core {}")
    }
}

struct PinnedCoreState {
    st_wait_mgr: SavestateWaitManager,
    es_wait_mgr: EmuStateWaitManager,
    core_handlers: HopSlotMap<StateHandlerKey, Box<dyn StateHandler>>
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
    pub fn init(
        path: impl AsRef<Path>,
        config_path: Option<&Path>,
        data_path: Option<&Path>,
    ) -> Result<Self, StartupError> {
        const CORE_DEBUG_ID: &CStr = c"m64p(core)";

        let mut guard = CORE_GUARD.lock().unwrap();
        if *guard {
            drop(guard);
            panic!("Only one instance of Core may be created");
        }

        let config_c_path = config_path.map(|path| CString::new(path.to_str().unwrap()).unwrap());
        let data_c_path = data_path.map(|path| CString::new(path.to_str().unwrap()).unwrap());

        // SAFETY: We assume that the path specified points to a valid Mupen64Plus core library.
        let api = unsafe { Container::<FullCoreApi>::load(path.as_ref()) }
            .map_err(StartupError::Library)?;

        let (st_tx, st_rx) = mpsc::channel();
        let (es_tx, es_rx) = mpsc::channel();

        let core = Self {
            plugins: None,
            pin_state: Box::new(Mutex::new(PinnedCoreState {
                st_wait_mgr: SavestateWaitManager::new(st_rx),
                es_wait_mgr: EmuStateWaitManager::new(es_rx),
                core_handlers: HopSlotMap::with_key(),
            })),
            // async waiters for state changes
            st_sender: st_tx,
            st_mutex: AsyncMutex::new(()),
            emu_sender: es_tx,
            // frontend hooks
            input_handler: None,
            audio_handler: None,
            api,
        };

        unsafe {
            // SAFETY: The core has yet to be initialized. The debug callback
            // context is an &'static CStr, and is guaranteed to outlive the core.
            // The state callback state is guaranteed to live at least as long
            // as the core due to the initialization order of this struct.
            core_fn(core.api.base.startup(
                0x02_01_00,
                config_c_path.as_ref().map_or(null(), |s| s.as_ptr()),
                data_c_path.as_ref().map_or(null(), |s| s.as_ptr()),
                CORE_DEBUG_ID.as_ptr() as *mut c_void,
                debug_callback,
                &*core.pin_state as *const Mutex<PinnedCoreState> as *mut c_void,
                state_callback,
            ))
            .map_err(StartupError::CoreInit)?;
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
    unsafe fn do_command_p(
        &self,
        command: Command,
        ptr_param: *mut c_void,
    ) -> Result<(), M64PError> {
        self.do_command_ip(command, 0, ptr_param)
    }

    #[inline(always)]
    fn do_command_i(&self, command: Command, int_param: c_int) -> Result<(), M64PError> {
        // SAFETY: Commands called in this form generally don't borrow any data.
        unsafe { self.do_command_ip(command, int_param, null_mut()) }
    }

    #[inline(always)]
    fn do_command(&self, command: Command) -> Result<(), M64PError> {
        // SAFETY: Commands called in this form generally don't borrow any data.
        unsafe { self.do_command_ip(command, 0, null_mut()) }
    }
}
impl Drop for Core {
    fn drop(&mut self) {
        // SAFETY: the core can be shut down at any time.
        unsafe { self.api.base.shutdown() };
    }
}

// Synchronous core commands
impl Core {
    /// Opens a ROM that is pre-loaded into memory.
    pub fn open_rom(&mut self, rom_data: &[u8]) -> Result<(), M64PError> {
        unsafe {
            // SAFETY: Mupen64Plus copies the ROM data passed into this function.
            // This means that it won't be invalidated if the ROM data borrowed here
            // goes out of scope.
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

    pub fn listen_state<F: StateHandler + 'static>(&mut self, f: F) -> StateHandlerKey {
        let mut pin_state = self.pin_state.lock().unwrap();
        pin_state.core_handlers.insert(Box::new(f))
    }

    pub fn unlisten_state(&mut self, handler: StateHandlerKey) {
        let mut pin_state = self.pin_state.lock().unwrap();
        pin_state.core_handlers.remove(handler);
    }

    /// Executes the currently-open ROM.
    pub fn execute(&self) -> Result<(), M64PError> {
        self.do_command(Command::Execute)
    }
}

static CORE_GUARD: Mutex<bool> = Mutex::new(false);

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
    let log_level =
        match MsgLevel::try_from(level as <MsgLevel as TryFromPrimitive>::Primitive).unwrap() {
            MsgLevel::Error => Level::Error,
            MsgLevel::Warning => Level::Warn,
            MsgLevel::Info => Level::Info,
            MsgLevel::Status => Level::Debug,
            MsgLevel::Verbose => Level::Trace,
            _ => panic!("Received invalid message level {}", level),
        };

    let target = CStr::from_ptr(context as *const c_char).to_str().unwrap();

    log!(target: target, log_level, "{}", CStr::from_ptr(message).to_str().unwrap());
}

#[allow(unused)]
extern "C" fn state_callback(context: *mut c_void, param: CoreParam, value: c_int) {
    let pinned_state_ref = unsafe { &*(context as *const Mutex<PinnedCoreState>) };
    let mut pinned_state = pinned_state_ref.lock().unwrap();

    log::debug!("state change: {:?} -> {:?}", param, value);

    for (_, mut callback) in &mut pinned_state.core_handlers {
        callback(param, value);
    }

    match param {
        CoreParam::StateSaveComplete | CoreParam::StateLoadComplete => {
            pinned_state.st_wait_mgr.on_state_change(param, value);
        }
        CoreParam::EmuState => {
            pinned_state.es_wait_mgr.on_state_change(value);
        }
        _ => (),
    }
}