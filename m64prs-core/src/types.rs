// Note: this is basically a port of m64p_types.h
use std::{
    error::Error,
    ffi::c_int,
    fmt::{Debug, Display},
};

include!(concat!(env!("OUT_DIR"), "/types.gen.rs"));

// ripped directly from core.
const M64P_ERROR_MESSAGES: [&str; 15] = [
    "SUCCESS: No error",
    "NOT_INIT: A function was called before it's associated module was initialized",
    "ALREADY_INIT: Initialization function called twice",
    "INCOMPATIBLE: API versions between components are incompatible",
    "INPUT_ASSERT: Invalid function parameters, such as a NULL pointer",
    "INPUT_INVALID: An input function parameter is logically invalid",
    "INPUT_NOT_FOUND: The input parameter(s) specified a particular item which was not found",
    "NO_MEMORY: Memory allocation failed",
    "FILES: Error opening, creating, reading, or writing to a file",
    "INTERNAL: logical inconsistency in program code.  Probably a bug.",
    "INVALID_STATE: An operation was requested which is not allowed in the current state",
    "PLUGIN_FAIL: A plugin function returned a fatal error",
    "SYSTEM_FAIL: A system function call, such as an SDL or file operation, failed",
    "UNSUPPORTED: Function call or argument is not supported (e.g. no debugger, invalid encoder format)",
    "WRONG_TYPE: A given input type parameter cannot be used for desired operation"
];

impl Display for m64p_error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for m64p_error {}
