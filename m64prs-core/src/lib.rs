mod core;
// pub mod ctypes;
pub mod error;
pub mod types;
mod macros;

pub use crate::core::{Core, Plugin};

#[cfg(test)]
mod tests {
    // use super::*;
}
