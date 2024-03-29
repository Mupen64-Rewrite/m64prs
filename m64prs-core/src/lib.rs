mod core;
// pub mod ctypes;
pub mod error;
mod macros;
pub mod reexports;
pub mod types;

pub use crate::core::{Core, Plugin};

#[cfg(test)]
mod tests {
    // use super::*;
}
