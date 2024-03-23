use std::ffi::c_uint;

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error, IntoPrimitive, TryFromPrimitive)]
pub enum M64PError {
    #[error("A function was called before it's associated module was initialized")]
    NotInit = m64prs_sys::Error::NotInit as u32,
    #[error("Initialization function called twice")]
    AlreadyInit = m64prs_sys::Error::AlreadyInit as u32,
    #[error("API versions between components are incompatible")]
    Incompatible = m64prs_sys::Error::Incompatible as u32,
    #[error("Invalid function parameters, such as a NULL pointer")]
    InputAssert = m64prs_sys::Error::InputAssert as u32,
    #[error("An input function parameter is logically invalid")]
    InputInvalid = m64prs_sys::Error::InputInvalid as u32,
    #[error("The input parameter(s) specified a particular item which was not found")]
    InputNotFound = m64prs_sys::Error::InputNotFound as u32,
    #[error("Memory allocation failed")]
    NoMemory = m64prs_sys::Error::NoMemory as u32,
    #[error("Error opening, creating, reading, or writing to a file")]
    Files = m64prs_sys::Error::Files as u32,
    #[error("Logical inconsistency in program code. Probably a bug.")]
    Internal = m64prs_sys::Error::Internal as u32,
    #[error("An operation was requested which is not allowed in the current state")]
    InvalidState = m64prs_sys::Error::InvalidState as u32,
    #[error("A plugin function returned a fatal error")]
    PluginFail = m64prs_sys::Error::PluginFail as u32,
    #[error("A system function call, such as an SDL or file operation, failed")]
    SystemFail = m64prs_sys::Error::SystemFail as u32,
    #[error(
        "Function call or argument is not supported (e.g. no debugger, invalid encoder format)"
    )]
    Unsupported = m64prs_sys::Error::Unsupported as u32,
    #[error("A given input type parameter cannot be used for desired operation")]
    WrongType = m64prs_sys::Error::WrongType as u32,
}

impl TryFrom<m64prs_sys::Error> for M64PError {
    type Error = TryFromPrimitiveError<M64PError>;

    fn try_from(value: m64prs_sys::Error) -> std::result::Result<Self, Self::Error> {
        let prim: u32 = value.into();
        prim.try_into()
    }
}

impl Into<m64prs_sys::Error> for M64PError {
    fn into(self) -> m64prs_sys::Error {
        let prim: u32 = self.into();
        prim.try_into().unwrap()
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
    #[error("The plugin is not valid for its type.")]
    PluginInvalid(m64prs_sys::PluginType),
    #[error("Could not convert {0} to a valid Mupen64Plus error")]
    ErrorConversionFailed(c_uint),
    #[error("Savestate loading failed, see logs for details")]
    LoadStateFailed,
    #[error("Savestate saving failed, see logs for details")]
    SaveStateFailed,
}

pub type Result<T> = ::std::result::Result<T, CoreError>;
