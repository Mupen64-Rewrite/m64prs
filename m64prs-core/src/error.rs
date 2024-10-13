use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

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


/// Error that may occur during initialization.
#[derive(Debug, Error)]
pub enum StartupError {
    /// An error occurred involving a dynamic library.
    #[error("Error occurred involving a dynamic library.")]
    Library(#[source] ::dlopen2::Error),
    /// An error occurred while initializing Mupen64Plus.
    #[error("Error occurred while initializing Mupen64Plus.")]
    CoreInit(#[source] M64PError),
}

#[derive(Debug, Error)]
pub enum SavestateError {
    /// An error occurred while requesting the savestate operation.
    #[error("Error occurred requesting the savestate operation.")]
    EarlyFail(#[source] M64PError),

    /// An error occurred while saving or loading the savestate.
    #[error("Error occurred while saving or loading the savestate")]
    SaveLoad
}

#[derive(Debug, Error)]
pub enum CoreError {
    /// An error occurred involving a dynamic library.
    #[error("Error occurred involving a dynamic library.")]
    Library(#[source] ::dlopen2::Error),
    /// An error occurred within Mupen64Plus or one of its plugins
    #[error("Error occurred within Mupen64Plus or one of its plugins.")]
    M64P(#[source] M64PError),
    /// An error occured during an I/O operation.
    #[error("Error occurred during an I/O operation.")]
    IO(#[source] ::std::io::Error),
    /// The plugin specified for a particular type isn't a valid plugin of that type.
    #[error("The plugin is not valid for its type.")]
    PluginInvalid(m64prs_sys::PluginType),
    /// A savestate load failed. Defails are likely logged.
    #[error("Savestate loading failed")]
    LoadStateFailed,
    /// A savestate save failed. Details are likely logged.
    #[error("Savestate saving failed")]
    SaveStateFailed,
}