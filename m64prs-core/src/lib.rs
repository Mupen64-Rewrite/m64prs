//! The core bindings for Mupen64Plus.
//! 
//! Contains the basic APIs for starting and using the Mupen64Plus(-rr) core.

mod core;
// pub mod ctypes;
pub mod error;
mod macros;
pub mod reexports;
pub mod types;

pub use crate::core::{Core, Plugin, ConfigSection};

#[cfg(test)]
mod tests {
    // use super::*;
}
