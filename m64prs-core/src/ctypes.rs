#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::{ffi::c_int, fmt::Display};
include!(concat!(env!("OUT_DIR"), "/types.gen.rs"));

// Miscellaneous extra trait implementations for the generated C bindings.

impl Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PluginType::CORE => f.write_str("core"),
            PluginType::GFX => f.write_str("graphics"),
            PluginType::AUDIO => f.write_str("audio"),
            PluginType::INPUT => f.write_str("input"),
            PluginType::RSP => f.write_str("RSP"),
            _ => f.write_str("(unknown)"),
        }
    }
}

impl Default for VideoExtensionFunctions {
    fn default() -> Self {
        Self {
            Functions: 17,
            VidExtFuncInit: None,
            VidExtFuncQuit: None,
            VidExtFuncListModes: None,
            VidExtFuncListRates: None,
            VidExtFuncSetMode: None,
            VidExtFuncSetModeWithRate: None,
            VidExtFuncGLGetProc: None,
            VidExtFuncGLSetAttr: None,
            VidExtFuncGLGetAttr: None,
            VidExtFuncGLSwapBuf: None,
            VidExtFuncSetCaption: None,
            VidExtFuncToggleFS: None,
            VidExtFuncResizeWindow: None,
            VidExtFuncGLGetDefaultFramebuffer: None,
            VidExtFuncInitWithRenderMode: None,
            VidExtFuncVKGetSurface: None,
            VidExtFuncVKGetInstanceExtensions: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VcrStartType(c_int);

impl VcrStartType {
    pub const FROM_SNAPSHOT: VcrStartType = VcrStartType(1 << 0);
    pub const FROM_START: VcrStartType = VcrStartType(1 << 1);
    pub const FROM_EEPROM: VcrStartType = VcrStartType(1 << 2);
}
