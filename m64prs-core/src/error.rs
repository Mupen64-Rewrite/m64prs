

use std::ffi::c_uint;

use thiserror::Error;

use crate::ctypes::{self, PluginType};


#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum M64PError {
    #[error("A function was called before it's associated module was initialized")]
    NotInit = ctypes::Error::NOT_INIT.0,
    #[error("Initialization function called twice")]
    AlreadyInit = ctypes::Error::ALREADY_INIT.0,
    #[error("API versions between components are incompatible")]
    Incompatible = ctypes::Error::INCOMPATIBLE.0,
    #[error("Invalid function parameters, such as a NULL pointer")]
    InputAssert = ctypes::Error::INPUT_ASSERT.0,
    #[error("An input function parameter is logically invalid")]
    InputInvalid = ctypes::Error::INPUT_INVALID.0,
    #[error("The input parameter(s) specified a particular item which was not found")]
    InputNotFound = ctypes::Error::INPUT_NOT_FOUND.0,
    #[error("Memory allocation failed")]
    NoMemory = ctypes::Error::NO_MEMORY.0,
    #[error("Error opening, creating, reading, or writing to a file")]
    Files = ctypes::Error::FILES.0,
    #[error("Logical inconsistency in program code. Probably a bug.")]
    Internal = ctypes::Error::INTERNAL.0,
    #[error("An operation was requested which is not allowed in the current state")]
    InvalidState = ctypes::Error::INVALID_STATE.0,
    #[error("A plugin function returned a fatal error")]
    PluginFail = ctypes::Error::PLUGIN_FAIL.0,
    #[error("A system function call, such as an SDL or file operation, failed")]
    SystemFail = ctypes::Error::SYSTEM_FAIL.0,
    #[error("Function call or argument is not supported (e.g. no debugger, invalid encoder format)")]
    Unsupported = ctypes::Error::UNSUPPORTED.0,
    #[error("A given input type parameter cannot be used for desired operation")]
    WrongType = ctypes::Error::WRONG_TYPE.0
}

impl TryFrom<ctypes::Error> for M64PError {
    type Error = CoreError;

    fn try_from(value: ctypes::Error) -> std::result::Result<Self, Self::Error> {
        match value {
            ctypes::Error::SUCCESS => Err(CoreError::InvalidEnumConversion),
            ctypes::Error::NOT_INIT => Ok(M64PError::NotInit),
            ctypes::Error::ALREADY_INIT => Ok(M64PError::AlreadyInit),
            ctypes::Error::INCOMPATIBLE => Ok(M64PError::Incompatible),
            ctypes::Error::INPUT_ASSERT => Ok(M64PError::InputAssert),
            ctypes::Error::INPUT_INVALID => Ok(M64PError::InputInvalid),
            ctypes::Error::INPUT_NOT_FOUND => Ok(M64PError::InputNotFound),
            ctypes::Error::NO_MEMORY => Ok(M64PError::NoMemory),
            ctypes::Error::FILES => Ok(M64PError::Files),
            ctypes::Error::INTERNAL => Ok(M64PError::Internal),
            ctypes::Error::INVALID_STATE => Ok(M64PError::InvalidState),
            ctypes::Error::PLUGIN_FAIL => Ok(M64PError::PluginFail),
            ctypes::Error::SYSTEM_FAIL => Ok(M64PError::SystemFail),
            ctypes::Error::UNSUPPORTED => Ok(M64PError::Unsupported),
            ctypes::Error::WRONG_TYPE => Ok(M64PError::WrongType),
            _ => Err(CoreError::InvalidEnumConversion)
        }
    }
}

impl Into<ctypes::Error> for M64PError {
    fn into(self) -> ctypes::Error {
        ctypes::Error(self as c_uint)
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
    #[error("This plugin is invalid")]
    InvalidPlugin,
    #[error("A {0} plugin was already attached.")]
    PluginAlreadyAttached(PluginType),
    #[error("There is no {0} plugin attached.")]
    NoPluginAttached(PluginType),
    #[error("INTERNAL: enum conversion failed.")]
    InvalidEnumConversion,
}

pub type Result<T> = ::std::result::Result<T, CoreError>;