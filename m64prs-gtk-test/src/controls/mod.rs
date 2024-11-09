use std::ffi::{c_int, c_void, CStr};

use gtk::{gdk, prelude::*};

pub(crate) trait PlatformSubsurface {
    fn swap_buffers(&mut self);
    fn set_position(&mut self, pos: dpi::PhysicalPosition<i32>);
    fn get_attribute(&mut self, attr: m64prs_sys::GLAttribute) -> c_int;
    fn resize_window(&mut self, size: dpi::PhysicalSize<u32>);
    fn set_visible(&mut self, visible: bool);
    fn get_proc_address(&mut self, symbol: &CStr) -> *mut c_void;
}

impl dyn PlatformSubsurface {
    pub fn new(window: gdk::Surface, size: dpi::PhysicalSize<u32>) -> Box<Self> {
        todo!()
    }
}