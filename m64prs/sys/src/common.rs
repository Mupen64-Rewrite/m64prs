//! Common safe types for Mupen64Plus.

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

/// Safer representation of a Mupen64Plus error which always represents an error state.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error, IntoPrimitive, TryFromPrimitive)]
pub enum M64PError {
    /// A function was called before it's associated module was initialized.
    #[error("A function was called before it's associated module was initialized")]
    NotInit = crate::Error::NotInit as u32,
    /// An initialization function was called twice.
    #[error("Initialization function called twice")]
    AlreadyInit = crate::Error::AlreadyInit as u32,
    /// API versions between components are incompatible
    #[error("API versions between components are incompatible")]
    Incompatible = crate::Error::Incompatible as u32,
    /// Invalid function parameter. Returned for trivial assertions, such as a pointer being non-null.
    #[error("Invalid function parameters, such as a NULL pointer")]
    InputAssert = crate::Error::InputAssert as u32,
    /// Invalid function parameter. Returned for non-trivial assertions.
    #[error("An input function parameter is logically invalid")]
    InputInvalid = crate::Error::InputInvalid as u32,
    /// A function parameter (such as a configuration key) was not found.
    #[error("The input parameter(s) specified a particular item which was not found")]
    InputNotFound = crate::Error::InputNotFound as u32,
    /// Memory allocation failed.
    #[error("Memory allocation failed")]
    NoMemory = crate::Error::NoMemory as u32,
    /// An error occurred when working with a file.
    #[error("Error opening, creating, reading, or writing to a file")]
    Files = crate::Error::Files as u32,
    /// An internal error.
    #[error("Logical inconsistency in program code. Probably a bug.")]
    Internal = crate::Error::Internal as u32,
    /// This function's module is not in the correct state to do this action.
    #[error("An operation was requested which is not allowed in the current state")]
    InvalidState = crate::Error::InvalidState as u32,
    /// A plugin caused a fatal error.
    #[error("A plugin function returned a fatal error")]
    PluginFail = crate::Error::PluginFail as u32,
    /// A system library caused an error.
    #[error("A system function call, such as an SDL or file operation, failed")]
    SystemFail = crate::Error::SystemFail as u32,
    /// The specified function is either unsupported or not compiled in.
    #[error(
        "Function call or argument is not supported (e.g. no debugger, invalid encoder format)"
    )]
    Unsupported = crate::Error::Unsupported as u32,
    /// A parameter representing a type is invalid for this operation.
    #[error("A given input type parameter cannot be used for desired operation")]
    WrongType = crate::Error::WrongType as u32,
}

impl TryFrom<crate::Error> for M64PError {
    type Error = TryFromPrimitiveError<M64PError>;

    fn try_from(value: crate::Error) -> std::result::Result<Self, Self::Error> {
        let prim = value as u32;
        prim.try_into()
    }
}

impl From<M64PError> for crate::Error {
    fn from(value: M64PError) -> Self {
        // SAFETY: every value of M64PError always corresponds to a value of crate::Error.
        unsafe { std::mem::transmute(value) }
    }
}