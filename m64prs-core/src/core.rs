use std::{
    ffi::{c_char, c_int, c_void},
    path::Path,
    sync::{Arc, LockResult, RwLock, RwLockReadGuard},
};

use dlopen2::wrapper::{Container, WrapperApi};
pub struct Core {
    api: Container<CoreApi>,
}

impl Core {
}

#[derive(WrapperApi)]
struct CoreApi {
    #[dlopen2_name = "CoreStartup"]
    startup: unsafe extern "C" fn(
        api_version: c_int,
        config_path: *const c_char,
        data_path: *const c_char,
        debug_context: *const c_void,
        debug_callback: unsafe extern "C" fn(
            context: *const c_void,
            level: c_int,
            message: *const c_char,
        ),
        state_context: *const c_void,
        state_callback: unsafe extern "C" fn(context: *const c_void, param: c_int, new_value: c_int),
    ),
    #[dlopen2_name = "CoreShutdown"]
    shutdown: unsafe extern "C" fn(),
}
