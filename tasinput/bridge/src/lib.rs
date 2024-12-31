use std::ffi::{c_char, c_int, c_uchar, c_void};

use common::M64PError;
use ext::FrontendInterfaceFFI;
use m64prs_sys::*;
use plugin_state::PluginState;
use std::sync::Mutex;

mod plugin_state;
mod util;

include!(concat!(env!("OUT_DIR"), "/version_gen.rs"));

static DUMMY_ADDRESS: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn PluginStartup(
    core_handle: DynlibHandle,
    debug_ctx: *mut c_void,
    debug_callback: ptr_DebugCallback,
) -> Error {
    check_state!(state uninit);

    let info = m64p_try!(
        decan::raw::get_address_info(&DUMMY_ADDRESS as *const i32 as *const _).ok_or_else(|| {
            log::error!("Failed to get own address");
            M64PError::SystemFail
        })
    );

    *state = Some(m64p_try!(PluginState::startup(
        &info.lib_path,
        core_handle,
        debug_ctx,
        debug_callback
    )));

    Error::Success
}

#[no_mangle]
pub unsafe extern "C" fn PluginShutdown() -> Error {
    check_state!(state init);
    *state = None;

    Error::Success
}

#[no_mangle]
pub unsafe extern "C" fn PluginGetVersion(
    plugin_type: *mut PluginType,
    plugin_version: *mut c_int,
    api_version: *mut c_int,
    plugin_name_ptr: *mut *const c_char,
    capabilities: *mut c_int,
) -> Error {
    if !plugin_type.is_null() {
        *plugin_type = PluginType::Input;
    }
    if !plugin_version.is_null() {
        *plugin_version = PLUGIN_VERSION;
    }
    if !api_version.is_null() {
        *api_version = API_VERSION;
    }
    if !plugin_name_ptr.is_null() {
        *plugin_name_ptr = PLUGIN_NAME.as_ptr();
    }
    if !capabilities.is_null() {
        *capabilities = 0;
    }

    Error::Success
}

#[no_mangle]
pub unsafe extern "C" fn ControllerCommand(control: c_int, command: *mut c_uchar) {}

#[no_mangle]
pub unsafe extern "C" fn GetKeys(control: c_int, keys: *mut Buttons) {
    with_init_state!(state => {
        *keys = state.get_keys(control as u8)
    });
}

#[no_mangle]
pub unsafe extern "C" fn InitiateControllers(info: ControlInfo) {
    with_init_state!(state => {
        state.init_controllers(info);
    });
}

#[no_mangle]
pub unsafe extern "C" fn ReadController(control: c_int, command: *mut c_uchar) {}

#[no_mangle]
pub unsafe extern "C" fn RomOpen() -> c_int {
    with_init_state!(state => {
        state.rom_open();
    });
    1
}

#[no_mangle]
pub unsafe extern "C" fn RomClosed() {
    with_init_state!(state => {
        state.rom_closed();
    });
}

#[no_mangle]
pub unsafe extern "C" fn SDL_KeyDown(sdl_mod: c_int, sdl_key: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn SDL_KeyUp(sdl_mod: c_int, sdl_key: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn M64PRS_UseFrontendInterface(pointer: *const FrontendInterfaceFFI) {
    // *FRONTEND.lock().unwrap() = Some((&*pointer).clone());
}

static STATE: Mutex<Option<PluginState>> = Mutex::new(None);
// static FRONTEND: Mutex<Option<FrontendInterfaceFFI>> = Mutex::new(None);

macro_rules! m64p_try {
    ($value:expr) => {
        match $value {
            Ok(value) => value,
            Err(error) => return error.into(),
        }
    };
}
macro_rules! check_state {
    ($state:ident init) => {
        let mut $state = STATE.lock().unwrap();
        if $state.is_none() {
            return Error::NotInit;
        }
    };
    ($state:ident uninit) => {
        let mut $state = STATE.lock().unwrap();
        if $state.is_some() {
            return Error::AlreadyInit;
        }
    };
}
macro_rules! with_init_state {
    ($state:ident => $content:expr) => {
        let mut $state = STATE.lock().unwrap();
        if let Some($state) = &mut *$state {
            $content
        }
    };
}
use {check_state, m64p_try, with_init_state};

// Static assertions on FFI signatures
const _: () = {
    const fn check_type_impl<T: Copy>(_: T) {}
    macro_rules! check_type {
        ($f:ident, $fp_ty:ty) => {
            check_type_impl::<$fp_ty>(Some($f));
        };
    }

    check_type!(PluginStartup, ptr_PluginStartup);
    check_type!(PluginShutdown, ptr_PluginShutdown);
    check_type!(PluginGetVersion, ptr_PluginGetVersion);
    check_type!(ControllerCommand, ptr_ControllerCommand);
    check_type!(GetKeys, ptr_GetKeys);
    check_type!(InitiateControllers, ptr_InitiateControllers);
    check_type!(ReadController, ptr_ReadController);
    check_type!(RomClosed, ptr_RomClosed);
    check_type!(RomOpen, ptr_RomOpen);
    check_type!(SDL_KeyDown, ptr_SDL_KeyDown);
    check_type!(SDL_KeyUp, ptr_SDL_KeyUp);
};
