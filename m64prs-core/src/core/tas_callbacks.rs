use std::{ffi::{c_int, c_void}, sync::OnceLock};

use m64prs_sys::Buttons;

use crate::core::PinnedCoreState;

use super::{core_fn, Core};

impl Core {
    pub fn set_input_filter(&mut self, callback: Box<dyn FnMut(u32, Buttons) -> Buttons + Send + Sync>) {
        static INIT_LOCK: OnceLock<()> = OnceLock::new();

        // SAFETY: PinnedCoreState is valid as long as the core is loaded. It's also
        // pinned, so its address remains stable over the core's lifetime.
        unsafe extern "C" fn call_input_filter(
            context: *mut c_void,
            port: c_int,
            input: *mut Buttons,
        ) {
            let pinned_state = unsafe { &mut *(context as *mut PinnedCoreState) };
            if let Some(ref mut filter) = pinned_state.input_filter_callback {
                *input = filter(port as u32, *input);
            }
        }

        INIT_LOCK.get_or_init(|| {
            core_fn(unsafe {
                self.api.tas.set_input_callback(
                    &mut *self.pin_state as *mut PinnedCoreState as *mut c_void,
                    Some(call_input_filter),
                )
            })
            .unwrap()
        });

        self.pin_state.input_filter_callback = Some(callback);
    }
    pub fn clear_input_filter(&mut self) {
        self.pin_state.input_filter_callback = None;
    }
}
