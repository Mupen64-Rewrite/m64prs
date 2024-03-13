#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::ffi::c_int;
include!(concat!(env!("OUT_DIR"), "/types.gen.rs"));

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VcrStartType(c_int);

impl VcrStartType {
    pub const FROM_SNAPSHOT: VcrStartType = VcrStartType(1 << 0);
    pub const FROM_START: VcrStartType = VcrStartType(1 << 1);
    pub const FROM_EEPROM: VcrStartType = VcrStartType(1 << 2);
}