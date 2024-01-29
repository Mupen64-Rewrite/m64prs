use std::{ffi::{c_char, c_int}, os::raw::c_void};

use dlopen2::wrapper::{Container, WrapperApi};

#[derive(WrapperApi)]
struct CoreApi {
    #[dlopen2_name="CoreStartup"]
    startup: unsafe extern "C" fn(
        api_version: c_int, 
        config_path: *const c_char, 
        data_path: *const c_char, 
        debug_context: *const c_void, 
        debug_callback: unsafe extern "C" fn(context: *const c_void, level: c_int, message: *const c_char),
        state_context: *const c_void,
        state_callback: unsafe extern "C" fn(context: *const c_void, param: c_int, new_value: c_int)
    )
}

const TEST_PATH: &str = "/home/jgcodes/Documents/Code/C++/mupen64plus-core-rr/projects/unix/libmupen64plus.so.2.0.0";

fn main() {
    let mut core: Container<CoreApi> = unsafe { Container::load(TEST_PATH) }.unwrap();
    println!("Core loaded!");
}
