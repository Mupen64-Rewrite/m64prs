use std::{error::Error, fmt::Display};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{ctypes::{self, m64p_error}, Core};

#[derive(Clone, Copy, Debug)]
pub struct InvalidEnumValue;

impl Display for InvalidEnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Provided integer does not correspond to an enum value")
    }
}

impl Error for InvalidEnumValue {}


#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
pub enum M64PError {
    NotInit = ctypes::M64ERR_NOT_INIT,
    AlreadyInit = ctypes::M64ERR_ALREADY_INIT,
    Incompatible = ctypes::M64ERR_INCOMPATIBLE,
    InputAssert = ctypes::M64ERR_INPUT_ASSERT,
    InputInvalid = ctypes::M64ERR_INPUT_INVALID,
    InputNotFound = ctypes::M64ERR_INPUT_NOT_FOUND,
    NoMemory = ctypes::M64ERR_NO_MEMORY,
    Files = ctypes::M64ERR_FILES,
    Internal = ctypes::M64ERR_INTERNAL,
    InvalidState = ctypes::M64ERR_INVALID_STATE,
    PluginFail = ctypes::M64ERR_PLUGIN_FAIL,
    SystemFail = ctypes::M64ERR_SYSTEM_FAIL,
    Unsupported = ctypes::M64ERR_UNSUPPORTED,
    WrongType = ctypes::M64ERR_WRONG_TYPE
}

impl Display for M64PError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let core = Core::get();
        f.write_str(core.get_error_message(self.clone()))
    }
}

impl Error for M64PError {}

impl TryFrom<m64p_error> for M64PError {
    type Error = InvalidEnumValue;

    fn try_from(value: m64p_error) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(InvalidEnumValue)
    }
}

#[derive(Debug)]
pub enum CoreError {
    /// An error occurred loading a dynamic library.
    Library(::dlopen2::Error),
    /// An error occurred within Mupen64Plus or a plugin.
    M64P(M64PError),
    /// Loaded plugin did not match the specified plugin type.
    PluginTypeNotMatching
}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Library(lib_err) => lib_err.fmt(f),
            CoreError::M64P(m64p_err) => m64p_err.fmt(f),
            CoreError::PluginTypeNotMatching => f.write_str("Loaded plugin doesn't match type"),
        }
    }
}

impl Error for CoreError {}