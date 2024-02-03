
mod ctypes;
mod core;
pub mod types;
pub mod error;

pub use crate::core::{Core, Plugin};
pub use crate::ctypes::{
    PluginType,
    Command
};


#[cfg(test)]
mod tests {
    // use super::*;
}
