use std::{
    ffi::{c_int, c_void},
    ptr::null_mut,
};

use m64prs_sys::Buttons;

use crate::{core::PinnedCoreState, error::M64PError};

use super::{core_fn, Core};

impl Core {
    pub fn set_input_filter(&mut self, callback: Box<dyn FnMut(u32, Buttons) -> Buttons>) {

        // SAFETY: PinnedCoreState is valid as long as the core is loaded. It's also
        // pinned, so its address remains stable over the core's lifetime.
        unsafe extern "C" fn call_input_filter(
            context: *mut c_void,
            port: c_int,
            input: *mut Buttons,
        ) {
            let pinned_state = unsafe { &mut *(context as *mut PinnedCoreState) };
            *input = pinned_state
                .input_filter_callback
                .as_mut()
                .map(|f| f(port as u32, *input))
                .unwrap_or(*input);
        }

        self.pin_state.input_filter_callback = Some(callback);
        core_fn(unsafe {
            self.api.tas.set_input_callback(
                &mut *self.pin_state as *mut PinnedCoreState as *mut c_void,
                Some(call_input_filter),
            )
        })
        .unwrap()
    }
    pub fn clear_input_filter(&mut self) {
        self.pin_state.input_filter_callback = None;
    }
}
