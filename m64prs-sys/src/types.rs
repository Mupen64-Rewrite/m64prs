#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
use std::{fmt::Display, hash::Hash, mem};

use bitflags::bitflags;

include!(concat!(env!("OUT_DIR"), "/types.gen.rs"));

impl Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ConfigType::Int => f.write_str("int"),
            ConfigType::Float => f.write_str("float"),
            ConfigType::Bool => f.write_str("bool"),
            ConfigType::String => f.write_str("string"),
        }
    }
}

// BUTTONS

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub struct ButtonFlags: u16 {
        const NONE = 0;

        const D_RIGHT = 1 << 0;
        const D_LEFT = 1 << 1;
        const D_DOWN = 1 << 2;
        const D_UP = 1 << 3;

        const START = 1 << 4;
        const Z = 1 << 5;
        const B = 1 << 6;
        const A = 1 << 7;

        const C_RIGHT = 1 << 8;
        const C_LEFT = 1 << 9;
        const C_DOWN = 1 << 10;
        const C_UP = 1 << 11;

        const R = 1 << 12;
        const L = 1 << 13;

        const RESERVED1 = 1 << 14;
        const RESERVED2 = 1 << 15;
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("The layout of `struct Buttons` has not been tested on this platform. Submit a PR if either the layout works or you can make it work.");

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, align(4))]
pub struct Buttons {
    pub button_bits: ButtonFlags,
    pub x_axis: i8,
    pub y_axis: i8,
}

impl Buttons {
    pub const BLANK: Buttons = Buttons { button_bits: ButtonFlags::NONE, x_axis: 0, y_axis: 0 };
}

impl From<u32> for Buttons {
    fn from(value: u32) -> Self {
        unsafe { mem::transmute(value) }
    }
}
impl From<Buttons> for u32 {
    fn from(value: Buttons) -> Self {
        unsafe { mem::transmute(value) }
    }
}
impl Hash for Buttons {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(u32::from(self.clone()))
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub struct VideoFlags: u32 {
        const SUPPORT_RESIZING = 1 << 1;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub struct CoreCaps: u32 {
        const DYNAREC = 1 << 0;
        const DEBUGGER = 1 << 1;
        const CORE_COMPARE = 1 << 2;
        const TAS_CALLBACKS = 1 << 16;
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::MaybeUninit, ptr::addr_of};

    use crate::{
        types::{Buttons, Error},
        ButtonFlags,
    };

    #[test]
    fn test_button_layout() {
        const UNINIT: MaybeUninit<Buttons> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();

        assert_eq!(std::mem::size_of::<Buttons>(), 4usize, "sizeof(Buttons)");
        assert_eq!(std::mem::align_of::<Buttons>(), 4usize, "alignof(Buttons)");

        unsafe { test_button_fields(ptr) };
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    unsafe fn test_button_fields(ptr: *const Buttons) {
        assert_eq!(
            addr_of!((*ptr).button_bits) as usize - ptr as usize,
            0usize,
            "offsetof(Buttons, button_bits)"
        );
        assert_eq!(
            addr_of!((*ptr).x_axis) as usize - ptr as usize,
            2usize,
            "offsetof(Buttons, x_axis)"
        );
        assert_eq!(
            addr_of!((*ptr).y_axis) as usize - ptr as usize,
            3usize,
            "offsetof(Buttons, y_axis)"
        );
    }

    #[test]
    fn test_error() {
        let err = Error::InputNotFound;
        let res: u32 = err.into();
        let _err2: Error = res.try_into().unwrap();
    }

    #[test]
    fn test_button_conversion() {
        assert_eq!(
            u32::from(Buttons {
                button_bits: ButtonFlags::NONE,
                x_axis: 127,
                y_axis: 127
            }),
            0x7F7F0000u32,
            "positive axes"
        );
        assert_eq!(
            u32::from(Buttons {
                button_bits: ButtonFlags::NONE,
                x_axis: -1,
                y_axis: -1
            }),
            0xFFFF0000u32,
            "negative axes"
        );
        assert_eq!(
            u32::from(Buttons {
                button_bits: ButtonFlags::Z | ButtonFlags::A | ButtonFlags::C_LEFT,
                x_axis: 0,
                y_axis: 0
            }),
            0x00002A0u32,
            "a few buttons"
        );
    }
}
