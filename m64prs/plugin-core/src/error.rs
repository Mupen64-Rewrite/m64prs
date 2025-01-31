pub use m64prs_sys::common::{M64PError, WrongConfigType};
use thiserror::Error;

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
