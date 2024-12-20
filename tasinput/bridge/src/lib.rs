use std::{ffi::{c_char, c_int, c_uchar, c_void}, ptr::NonNull};

use m64prs_sys::*;

mod state;

include!(concat!(env!("OUT_DIR"), "/version_gen.rs"));

#[no_mangle]
pub unsafe extern "C" fn PluginStartup(
    core_handle: DynlibHandle,
    debug_ctx: *mut c_void,
    debug_callback: ptr_DebugCallback,
) -> Error {
    Error::Success
}

#[no_mangle]
pub unsafe extern "C" fn PluginShutdown() -> Error {
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
    *keys = Buttons::BLANK;
}

#[no_mangle]
pub unsafe extern "C" fn InitiateControllers(info: ControlInfo) {
    
}

#[no_mangle]
pub unsafe extern "C" fn ReadController(control: c_int, command: *mut c_uchar) {}

#[no_mangle]
pub unsafe extern "C" fn RomOpen() -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn RomClosed() {}

#[no_mangle]
pub unsafe extern "C" fn SDL_KeyDown(sdl_mod: c_int, sdl_key: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn SDL_KeyUp(sdl_mod: c_int, sdl_key: c_int) {}

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
