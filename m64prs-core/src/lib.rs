mod core;
// pub mod ctypes;
pub mod error;
mod macros;
pub mod types;

pub use crate::core::{Core, Plugin};

#[cfg(test)]
mod tests {
    // use super::*;
}
