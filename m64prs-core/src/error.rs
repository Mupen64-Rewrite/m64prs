use std::{error::Error, fmt::Display};

use thiserror::Error;

use crate::ctypes::{self, m64p_error};


#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum M64PError {
    #[error("A function was called before it's associated module was initialized")]
    NotInit = ctypes::M64ERR_NOT_INIT,
    #[error("Initialization function called twice")]
    AlreadyInit = ctypes::M64ERR_ALREADY_INIT,
    #[error("API versions between components are incompatible")]
    Incompatible = ctypes::M64ERR_INCOMPATIBLE,
    #[error("Invalid function parameters, such as a NULL pointer")]
    InputAssert = ctypes::M64ERR_INPUT_ASSERT,
    #[error("An input function parameter is logically invalid")]
    InputInvalid = ctypes::M64ERR_INPUT_INVALID,
    #[error("The input parameter(s) specified a particular item which was not found")]
    InputNotFound = ctypes::M64ERR_INPUT_NOT_FOUND,
    #[error("Memory allocation failed")]
    NoMemory = ctypes::M64ERR_NO_MEMORY,
    #[error("Error opening, creating, reading, or writing to a file")]
    Files = ctypes::M64ERR_FILES,
    #[error("Logical inconsistency in program code. Probably a bug.")]
    Internal = ctypes::M64ERR_INTERNAL,
    #[error("An operation was requested which is not allowed in the current state")]
    InvalidState = ctypes::M64ERR_INVALID_STATE,
    #[error("A plugin function returned a fatal error")]
    PluginFail = ctypes::M64ERR_PLUGIN_FAIL,
    #[error("A system function call, such as an SDL or file operation, failed")]
    SystemFail = ctypes::M64ERR_SYSTEM_FAIL,
    #[error("Function call or argument is not supported (e.g. no debugger, invalid encoder format)")]
    Unsupported = ctypes::M64ERR_UNSUPPORTED,
    #[error("A given input type parameter cannot be used for desired operation")]
    WrongType = ctypes::M64ERR_WRONG_TYPE
}

impl From<m64p_error> for M64PError {
    fn from(value: m64p_error) -> Self {
        match value {
            ctypes::M64ERR_SUCCESS => panic!("Refusing to convert M64ERR_SUCCESS to an error type"),
            ctypes::M64ERR_NOT_INIT => M64PError::NotInit,
            ctypes::M64ERR_ALREADY_INIT => M64PError::AlreadyInit,
            ctypes::M64ERR_INCOMPATIBLE => M64PError::Incompatible,
            ctypes::M64ERR_INPUT_ASSERT => M64PError::InputAssert,
            ctypes::M64ERR_INPUT_INVALID => M64PError::InputInvalid,
            ctypes::M64ERR_INPUT_NOT_FOUND => M64PError::InputNotFound,
            ctypes::M64ERR_NO_MEMORY => M64PError::NoMemory,
            ctypes::M64ERR_FILES => M64PError::Files,
            ctypes::M64ERR_INTERNAL => M64PError::Internal,
            ctypes::M64ERR_INVALID_STATE => M64PError::InvalidState,
            ctypes::M64ERR_PLUGIN_FAIL => M64PError::PluginFail,
            ctypes::M64ERR_SYSTEM_FAIL => M64PError::SystemFail,
            ctypes::M64ERR_UNSUPPORTED => M64PError::Unsupported,
            ctypes::M64ERR_WRONG_TYPE => M64PError::WrongType,
            _ => panic!("Unknown Mupen64Plus error type.")
        }
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Error occurred involving a dynamic library.")]
    Library(#[source] ::dlopen2::Error),
    #[error("Error occurred within Mupen64Plus or one of its plugins.")]
    M64P(#[source] M64PError),
    #[error("Error occurred when performing I/O.")]
    IO(#[source] ::std::io::Error),
    #[error("Loaded plugin did not match the specified plugin type.")]
    PluginTypeNotMatching,
    #[error("INTERNAL: enum conversion failed.")]
    InvalidEnumConversion,

}

pub type Result<T> = ::std::result::Result<T, CoreError>;