use std::fmt::Display;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{ctypes::{self, m64p_plugin_type}, error::InvalidEnumValue};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
pub enum PluginType {
    RSP = ctypes::M64PLUGIN_RSP,
    Graphics = ctypes::M64PLUGIN_GFX,
    Audio = ctypes::M64PLUGIN_AUDIO,
    Input = ctypes::M64PLUGIN_INPUT
}

impl Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl TryFrom<m64p_plugin_type> for PluginType {
    type Error = InvalidEnumValue;

    fn try_from(value: m64p_plugin_type) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(InvalidEnumValue)
    }
}