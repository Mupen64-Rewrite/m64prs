use std::{
    error::Error,
    fmt::{Debug, Display},
};

use m64prs_sys::ConfigType;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

use crate::plugin::PluginType;

/// Safer representation of a Mupen64Plus error which always represents an error state.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error, IntoPrimitive, TryFromPrimitive)]
pub enum M64PError {
    /// A function was called before it's associated module was initialized.
    #[error("A function was called before it's associated module was initialized")]
    NotInit = m64prs_sys::Error::NotInit as u32,
    /// An initialization function was called twice.
    #[error("Initialization function called twice")]
    AlreadyInit = m64prs_sys::Error::AlreadyInit as u32,
    /// API versions between components are incompatible
    #[error("API versions between components are incompatible")]
    Incompatible = m64prs_sys::Error::Incompatible as u32,
    /// Invalid function parameter. Returned for trivial assertions, such as a pointer being non-null.
    #[error("Invalid function parameters, such as a NULL pointer")]
    InputAssert = m64prs_sys::Error::InputAssert as u32,
    /// Invalid function parameter. Returned for non-trivial assertions.
    #[error("An input function parameter is logically invalid")]
    InputInvalid = m64prs_sys::Error::InputInvalid as u32,
    /// A function parameter (such as a configuration key) was not found.
    #[error("The input parameter(s) specified a particular item which was not found")]
    InputNotFound = m64prs_sys::Error::InputNotFound as u32,
    /// Memory allocation failed.
    #[error("Memory allocation failed")]
    NoMemory = m64prs_sys::Error::NoMemory as u32,
    /// An error occurred when working with a file.
    #[error("Error opening, creating, reading, or writing to a file")]
    Files = m64prs_sys::Error::Files as u32,
    /// An internal error.
    #[error("Logical inconsistency in program code. Probably a bug.")]
    Internal = m64prs_sys::Error::Internal as u32,
    /// This function's module is not in the correct state to do this action.
    #[error("An operation was requested which is not allowed in the current state")]
    InvalidState = m64prs_sys::Error::InvalidState as u32,
    /// A plugin caused a fatal error.
    #[error("A plugin function returned a fatal error")]
    PluginFail = m64prs_sys::Error::PluginFail as u32,
    /// A system library caused an error.
    #[error("A system function call, such as an SDL or file operation, failed")]
    SystemFail = m64prs_sys::Error::SystemFail as u32,
    /// The specified function is either unsupported or not compiled in.
    #[error(
        "Function call or argument is not supported (e.g. no debugger, invalid encoder format)"
    )]
    Unsupported = m64prs_sys::Error::Unsupported as u32,
    /// A parameter representing a type is invalid for this operation.
    #[error("A given input type parameter cannot be used for desired operation")]
    WrongType = m64prs_sys::Error::WrongType as u32,
}

impl TryFrom<m64prs_sys::Error> for M64PError {
    type Error = TryFromPrimitiveError<M64PError>;

    fn try_from(value: m64prs_sys::Error) -> std::result::Result<Self, Self::Error> {
        let prim = value as u32;
        prim.try_into()
    }
}

impl From<M64PError> for m64prs_sys::Error {
    fn from(value: M64PError) -> Self {
        // SAFETY: every value of M64PError always corresponds to a value of m64prs_sys::Error.
        unsafe { std::mem::transmute(value) }
    }
}

/// Error that may occur during initialization.
#[derive(Debug, Error)]
pub enum StartupError {
    /// An error occurred involving a dynamic library.
    #[error("dynamic library load failed")]
    Library(#[source] ::dlopen2::Error),
    /// An error occurred while initializing Mupen64Plus.
    #[error("Mupen64Plus startup failed")]
    CoreInit(#[source] M64PError),
}

/// Error that may occur during a savestate save or load.
#[derive(Debug, Error)]
pub enum SavestateError {
    /// An error occurred while requesting the savestate operation.
    #[error("core command failed immediately: {0}")]
    EarlyFail(#[source] M64PError),

    /// An error occurred while saving or loading the savestate.
    #[error("savestate save/load failed")]
    SaveLoad,
}

/// Error that may occur during plugin loading.
#[derive(Debug, Error)]
pub enum PluginLoadError {
    /// An error occurred involving a dynamic library.
    #[error("dynamic library raised error")]
    Library(#[source] ::dlopen2::Error),
    /// An error occurred within Mupen64Plus or one of its plugins
    #[error("plugin function raised error")]
    M64P(#[source] M64PError),
    /// The plugin specified for a particular type isn't a valid plugin of that type.
    #[error("{0:?} is an invalid plugin type for this type")]
    InvalidType(m64prs_sys::PluginType),
}

#[derive(Debug)]
pub struct WrongPluginType {
    expected: PluginType,
    actual: PluginType,
}

impl WrongPluginType {
    pub fn new(expected: PluginType, actual: PluginType) -> Self {
        Self { expected, actual }
    }
}

impl Display for WrongPluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Plugin type is {} (expected {})",
            self.actual, self.expected
        ))
    }
}

impl Error for WrongPluginType {}

#[derive(Debug)]
pub struct WrongConfigType {
    expected: ConfigType,
    actual: ConfigType,
}

impl WrongConfigType {
    pub fn new(expected: ConfigType, actual: ConfigType) -> Self {
        Self { expected, actual }
    }
}

impl Display for WrongConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "parameter type is {} (expected {})",
            self.actual, self.expected
        ))
    }
}

impl Error for WrongConfigType {}
