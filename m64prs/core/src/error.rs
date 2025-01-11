use std::fmt::Debug;

use thiserror::Error;

use crate::plugin::PluginType;

pub use m64prs_sys::common::{M64PError, WrongConfigType};

/// Error that may occur during initialization.
#[derive(Debug, Error)]
pub enum StartupError {
    /// An error occurred involving a dynamic library.
    #[error("dynamic library load failed: {0}")]
    Library(#[source] decan::LoadOrSymbolGroupError),
    /// An error occurred while initializing Mupen64Plus.
    #[error("Mupen64Plus startup failed: {0}")]
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
    #[error("dynamic library raised error: {0}")]
    Library(#[source] decan::LoadOrSymbolGroupError),
    /// An error occurred within Mupen64Plus or one of its plugins
    #[error("plugin function raised error: {0}")]
    M64P(#[source] M64PError),
    /// The plugin specified for a particular type isn't a valid plugin of that type.
    #[error("{0:?} is an invalid plugin type for this type")]
    InvalidType(m64prs_sys::PluginType),
}

/// Error that occurs because the plugin is not the type expected when converting.
#[derive(Debug, Error)]
#[error("Plugin type is {actual} (expected {expected}")]
pub struct WrongPluginType {
    expected: PluginType,
    actual: PluginType,
}

impl WrongPluginType {
    pub fn new(expected: PluginType, actual: PluginType) -> Self {
        Self { expected, actual }
    }
}

#[derive(Debug, Error)]
pub enum ConfigGetError {
    /// An error occurred within Mupen64Plus or one of its plugins
    #[error("M64+ error: {0}")]
    M64P(#[source] M64PError),
    #[error("{0}")]
    WrongConfigType(#[source] WrongConfigType),
}

impl From<M64PError> for ConfigGetError {
    fn from(value: M64PError) -> Self {
        Self::M64P(value)
    }
}
impl From<WrongConfigType> for ConfigGetError {
    fn from(value: WrongConfigType) -> Self {
        Self::WrongConfigType(value)
    }
}