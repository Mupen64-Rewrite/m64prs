use std::{ffi::c_int, fmt::Display};

use crate::{ctypes::{self, m64p_plugin_type, M64PLUGIN_AUDIO, M64PLUGIN_GFX, M64PLUGIN_INPUT, M64PLUGIN_RSP}, error::CoreError};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginType {
    RSP = ctypes::M64PLUGIN_RSP,
    Video = ctypes::M64PLUGIN_GFX,
    Audio = ctypes::M64PLUGIN_AUDIO,
    Input = ctypes::M64PLUGIN_INPUT
}

impl Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl TryFrom<m64p_plugin_type> for PluginType {
    type Error = CoreError;

    fn try_from(value: m64p_plugin_type) -> Result<Self, Self::Error> {
        match value {
            ctypes::M64PLUGIN_RSP => Ok(Self::RSP),
            ctypes::M64PLUGIN_GFX => Ok(Self::Video),
            ctypes::M64PLUGIN_AUDIO => Ok(Self::Audio),
            ctypes::M64PLUGIN_INPUT => Ok(Self::Input),
            _ => Err(CoreError::InvalidEnumConversion)
        }
    }
}

impl Into<m64p_plugin_type> for PluginType {
    fn into(self) -> m64p_plugin_type {
        self as m64p_plugin_type
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum APIType {
    Core = ctypes::M64PLUGIN_CORE,
    Plugin(PluginType)
}

impl TryFrom<m64p_plugin_type> for APIType {
    type Error = CoreError;

    fn try_from(value: m64p_plugin_type) -> Result<Self, Self::Error> {
        if let Ok(plugin_type) = value.try_into() {
            return Ok(APIType::Plugin(plugin_type));
        }
        else if value == ctypes::M64PLUGIN_CORE {
            return Ok(APIType::Core);
        }
        else {
            return Err(CoreError::InvalidEnumConversion)
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct APIVersion {
    pub api_type: APIType,
    pub plugin_version: c_int,
    pub api_version: c_int,
    pub plugin_name: &'static str,
    pub capabilities: c_int
}