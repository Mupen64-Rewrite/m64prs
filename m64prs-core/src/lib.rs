//! The core bindings for Mupen64Plus.
//!
//! Contains the basic APIs for starting and using the Mupen64Plus(-rr) core.

pub mod core;
// pub mod ctypes;
pub mod error;
pub mod reexports;

pub use crate::core::*;

#[cfg(test)]
mod tests {
    // use super::*;
}
