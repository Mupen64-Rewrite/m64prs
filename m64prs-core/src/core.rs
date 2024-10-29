use std::{
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    fmt::Debug,
    path::Path,
    pin::Pin,
    ptr::{null, null_mut},
    sync::{mpsc, Mutex},
};

use async_std::sync::Mutex as AsyncMutex;
use dlopen2::wrapper::Container;
use emu_state::{EmulatorWaitManager, EmulatorWaiter};
use log::{log, Level};
use num_enum::TryFromPrimitive;

use crate::error::{M64PError, StartupError};

use m64prs_sys::{
    api::FullCoreApi, Buttons, Command, CoreParam, MsgLevel
};

use self::save::{SavestateWaitManager, SavestateWaiter};

mod config;
mod emu_state;
mod plugin;
mod save;
mod tas_callbacks;
mod vidext;

pub use plugin::Plugin;
pub use config::ConfigSection;

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
    let log_level = match MsgLevel::try_from(level as <MsgLevel as TryFromPrimitive>::Primitive).unwrap() {
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
    let pinned_state = unsafe { &mut *(context as *mut PinnedCoreState) };

    match param {
        CoreParam::StateSaveComplete | CoreParam::StateLoadComplete => {
            pinned_state.st_wait_mgr.on_state_change(param, value);
        }
        CoreParam::EmuState => {
            pinned_state.emu_wait_mgr.on_emu_state_changed(value);
        }
        _ => (),
    }
}

struct PinnedCoreState {
    st_wait_mgr: SavestateWaitManager,
    emu_wait_mgr: EmulatorWaitManager,
    input_filter_callback: Option<Box<dyn FnMut(u32, Buttons) -> Buttons + Send + Sync>>
}

static CORE_GUARD: Mutex<bool> = Mutex::new(false);

pub struct Core {
    pin_state: Pin<Box<PinnedCoreState>>,

    api: Container<FullCoreApi>,
    plugins: Option<[Plugin; 4]>,
    
    save_sender: mpsc::Sender<SavestateWaiter>,
    save_mutex: AsyncMutex<()>,

    emu_sender: mpsc::Sender<EmulatorWaiter>,
    emu_mutex: AsyncMutex<()>
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
        const CORE_DEBUG_ID: &'static CStr = c"m64p(core)";

        let mut guard = CORE_GUARD.lock().unwrap();
        if *guard {
            drop(guard);
            panic!("Only one instance of Core may be created");
        }

        let api = unsafe { Container::<FullCoreApi>::load(path.as_ref()) }
            .map_err(StartupError::Library)?;

        let (save_tx, save_rx) = mpsc::channel();
        let (emu_tx, emu_rx) = mpsc::channel();

        let mut core = Self {
            api,
            plugins: None,
            pin_state: Box::pin(PinnedCoreState {
                st_wait_mgr: SavestateWaitManager::new(save_rx),
                emu_wait_mgr: EmulatorWaitManager::new(emu_rx),
                input_filter_callback: None,
            }),
            save_sender: save_tx,
            save_mutex: AsyncMutex::new(()),
            
            emu_sender: emu_tx,
            emu_mutex: AsyncMutex::new(())
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
            ))
            .map_err(|err| StartupError::CoreInit(err))?;
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

    pub fn set_frame_callback(
        &mut self,
        callback: unsafe extern "C" fn(c_uint),
    ) -> Result<(), M64PError> {
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