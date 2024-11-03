use std::{
    ffi::{c_int, c_uint, c_void},
    sync::OnceLock,
};

use ffi::InputHandlerFFI;
use m64prs_sys::Buttons;

use crate::error::M64PError;

use super::{core_fn, Core};

impl Core {
    pub fn set_input_handler<I: InputHandler>(&mut self, handler: I) -> Result<(), M64PError> {
        let mut input_handler = InputHandlerFFI::new(handler);

        core_fn(unsafe { self.api.tas.set_input_handler(&mut input_handler.create_ffi_handler()) })?;
        let _ = self.input_handler.replace(Box::new(input_handler));

        Ok(())
    }


}

pub trait InputHandler: Send + 'static {
    fn filter_inputs(&mut self, port: c_int, input: Buttons) -> Buttons;
    fn poll_present(&mut self, port: c_int) -> bool;
}

pub trait AudioHandler: Send + 'static {
    fn set_audio_rate(new_rate: c_uint);
    fn push_audio_samples(data: &[u16]);
}

pub trait SaveHandler: Send + 'static {
    const SIGNATURE: u32;
    const VERSION: u32;
    const ALLOC_SIZE: usize;

    fn save_extra_data(&mut self, data: &mut [u8]);
    fn load_extra_data(&mut self, version: u32, data: &[u8]);
    fn get_data_size(&mut self, version: u32) -> usize;
}

pub mod ffi {

    use std::mem;
    use m64prs_sys::Error as SysError;

    use super::*;

    pub(crate) struct InputHandlerFFI<I: InputHandler>(*mut I);

    unsafe impl<I: InputHandler> Send for InputHandlerFFI<I> {}

    impl<I: InputHandler> InputHandlerFFI<I> {
        pub(super) fn new(handler: I) -> Self {
            let heap_alloc = Box::into_raw(Box::new(handler));
            Self(heap_alloc)
        }

        pub(super) fn create_ffi_handler(&mut self) -> m64prs_sys::TasInputHandler {
            m64prs_sys::TasInputHandler {
                context: self.0 as *mut c_void,
                filter_inputs: Some(Self::ffi_filter_inputs),
                poll_present: Some(Self::ffi_poll_present),
            }
        }

        unsafe extern "C" fn ffi_filter_inputs(
            context: *mut c_void,
            port: c_int,
            input: *mut Buttons,
        ) {
            let context = context as *mut I;
            *input = (&mut *context).filter_inputs(port, *input);
        }

        unsafe extern "C" fn ffi_poll_present(
            context: *mut c_void,
            port: c_int,
        ) -> bool {
            let context = context as *mut I;
            (&mut *context).poll_present(port)
        }
    }

    impl<I: InputHandler> Drop for InputHandlerFFI<I> {
        fn drop(&mut self) {
            mem::drop(unsafe { Box::from_raw(self.0) });
        }
    }
}
