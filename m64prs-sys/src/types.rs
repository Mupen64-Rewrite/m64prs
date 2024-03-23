use std::mem;

use bitflags::bitflags;

include!(concat!(env!("OUT_DIR"), "/types.gen.rs"));

// BUTTONS

#[cfg(any(target_arch = "x86_64"))]
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub struct ButtonFlags: u16 {
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

#[cfg(any(target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, align(4))]
pub struct Buttons {
    pub button_bits: ButtonFlags,
    pub x_axis: i8,
    pub y_axis: i8,
}

#[cfg(not(any(target_arch = "x86_64")))]
compile_error!("The layout of `struct Buttons` has not been tested on this platform. Submit a PR if either the layout works or you can make it work.");

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

// VCR start type

/*
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VcrStartType(c_int);

impl VcrStartType {
    pub const FROM_SNAPSHOT: VcrStartType = VcrStartType(1 << 0);
    pub const FROM_START: VcrStartType = VcrStartType(1 << 1);
    pub const FROM_EEPROM: VcrStartType = VcrStartType(1 << 2);
}
*/
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub struct VcrStartType: u32 {
        const FROM_SNAPSHOT = 1 << 0;
        const FROM_START = 1 << 1;
        const FROM_EEPROM = 1 << 2;
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::MaybeUninit, ptr::addr_of};

    use crate::types::{Buttons, Error};

    #[test]
    fn test_button_layout() {
        const UNINIT: MaybeUninit<Buttons> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();

        assert_eq!(std::mem::size_of::<Buttons>(), 4usize, "sizeof(Buttons)");
        assert_eq!(std::mem::align_of::<Buttons>(), 4usize, "alignof(Buttons)");

        unsafe { test_button_fields(ptr) };
    }

    #[cfg(any(target_arch = "x86_64"))]
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
}
