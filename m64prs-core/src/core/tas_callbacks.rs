use ffi::{AudioHandlerFFI, InputHandlerFFI, SaveHandlerFFI};
use m64prs_sys::{Buttons, Command};
use std::ffi::{c_int, c_uint, c_void};

use crate::error::M64PError;

use super::{core_fn, Core};

impl Core {
    /// Sets an *input handler* for the core, which can filter or replace controller inputs.
    /// It may only be set once.
    ///
    /// # Errors
    /// This function errors if the core fails to set the input handler.
    ///
    /// # Panics
    /// This function panics if the input handler is already set.
    pub fn set_input_handler<I: InputHandler>(&mut self, handler: I) -> Result<(), M64PError> {
        if self.input_handler.is_some() {
            panic!("input handler already registered");
        }
        let input_handler = InputHandlerFFI::new(handler);

        // SAFETY: the FFI handler is safe to use as long as the context isn't moved.
        core_fn(unsafe {
            self.api
                .tas
                .set_input_handler(&input_handler.create_ffi_handler())
        })?;
        // This reference keeps the context from being moved or deleted.
        self.input_handler = Some(Box::new(input_handler));

        Ok(())
    }

    /// Sets a *frame handler* for the core, which executes a callback when
    /// a new frame is presented to the screen.
    ///
    /// # Errors
    /// This function errors if the core fails to set the frame handler.
    ///
    /// # Panics
    /// The function errors if the frame handler is already set.
    pub fn set_frame_handler<F: FrameHandler>(&mut self, handler: F) -> Result<(), M64PError> {
        static mut FRAME_HANDLER_BOX: Option<Box<dyn FrameHandler>> = None;

        // SAFETY: There is only ever one instance of Core, and so, there can only
        // ever be one caller at any given time.
        unsafe {
            if FRAME_HANDLER_BOX.is_some() {
                panic!("frame handler already registered");
            }

            FRAME_HANDLER_BOX = Some(Box::new(handler));
        }

        unsafe extern "C" fn frame_handler(count: c_uint) {
            FRAME_HANDLER_BOX.as_mut().unwrap().new_frame(count);
        }

        // SAFETY: the frame handler is valid as long as the core is.
        unsafe { self.do_command_p(Command::SetFrameCallback, frame_handler as *mut _) }
    }

    /// Sets an *audio handler* for the core, which can receive and process audio
    /// samples secondary to the audio plugin.
    ///
    /// # Errors
    /// This function errors if the core fails to set the audio handler.
    ///
    /// # Panics
    /// The function errors if the audio handler is already set.
    pub fn set_audio_handler<A: AudioHandler>(&mut self, handler: A) -> Result<(), M64PError> {
        if self.audio_handler.is_some() {
            panic!("audio handler already registered");
        }

        let audio_handler = AudioHandlerFFI::new(handler);

        // SAFETY: This works the exact same way as input_handler.
        core_fn(unsafe {
            self.api
                .tas
                .set_audio_handler(&audio_handler.create_ffi_handler())
        })?;
        self.audio_handler = Some(Box::new(audio_handler));

        Ok(())
    }

    /// Sets a *save handler* for the core, which can save and load
    /// extra data appended onto savestates.
    ///
    /// # Errors
    /// This function errors if the core fails to set the save handler.
    ///
    /// # Panics
    /// The function errors if the save handler is already set.
    pub fn set_save_handler<S: SaveHandler>(&mut self, handler: S) -> Result<(), M64PError> {
        if self.save_handler.is_some() {
            panic!("save handler already registered");
        }

        let save_handler = SaveHandlerFFI::new(handler);
        
        // SAFETY: This also works the same way as input_handler.
        core_fn(unsafe {
            self.api.tas.set_savestate_handler(&save_handler.create_ffi_handler())
        })?;
        self.save_handler = Some(Box::new(save_handler));

        Ok(())
    }
}

pub trait InputHandler: Send + 'static {
    fn filter_inputs(&mut self, port: c_int, input: Buttons) -> Buttons;
    fn poll_present(&mut self, port: c_int) -> bool;
}

pub trait AudioHandler: Send + 'static {
    fn set_audio_rate(&mut self, new_rate: c_uint);
    fn push_audio_samples(&mut self, data: &[u16]);
}

pub trait SaveHandler: Send + 'static {
    const SIGNATURE: u32;
    const VERSION: u32;
    const ALLOC_SIZE: usize;

    fn save_extra_data(&mut self, data: &mut [u8]);
    fn load_extra_data(&mut self, version: u32, data: &[u8]);
    fn get_data_size(&mut self, version: u32) -> usize;
}

pub trait FrameHandler: Send + 'static {
    fn new_frame(&mut self, count: c_uint);
}

pub mod ffi {
    use super::*;
    use std::{ffi::c_char, mem};

    pub(crate) trait FFIHandler: Send {}

    pub(crate) struct InputHandlerFFI<I: InputHandler>(*mut I);

    unsafe impl<I: InputHandler> Send for InputHandlerFFI<I> {}
    impl<I: InputHandler> FFIHandler for InputHandlerFFI<I> {}

    impl<I: InputHandler> InputHandlerFFI<I> {
        pub(super) fn new(handler: I) -> Self {
            let heap_alloc = Box::into_raw(Box::new(handler));
            Self(heap_alloc)
        }

        pub(super) unsafe fn create_ffi_handler(&self) -> m64prs_sys::TasInputHandler {
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
            *input = (*context).filter_inputs(port, *input);
        }

        unsafe extern "C" fn ffi_poll_present(context: *mut c_void, port: c_int) -> bool {
            let context = context as *mut I;
            (*context).poll_present(port)
        }
    }

    impl<I: InputHandler> Drop for InputHandlerFFI<I> {
        fn drop(&mut self) {
            mem::drop(unsafe { Box::from_raw(self.0) });
        }
    }

    pub(crate) struct AudioHandlerFFI<A: AudioHandler>(*mut A);

    unsafe impl<A: AudioHandler> Send for AudioHandlerFFI<A> {}
    impl<A: AudioHandler> FFIHandler for AudioHandlerFFI<A> {}

    impl<A: AudioHandler> AudioHandlerFFI<A> {
        pub(super) fn new(handler: A) -> Self {
            let heap_alloc = Box::into_raw(Box::new(handler));
            Self(heap_alloc)
        }

        pub(super) unsafe fn create_ffi_handler(&self) -> m64prs_sys::TasAudioHandler {
            m64prs_sys::TasAudioHandler {
                context: self.0 as *mut c_void,
                set_audio_rate: Some(Self::ffi_set_audio_rate),
                push_audio_samples: Some(Self::ffi_push_audio_samples),
            }
        }

        unsafe extern "C" fn ffi_set_audio_rate(context: *mut c_void, new_rate: u32) {
            let context = context as *mut A;
            (*context).set_audio_rate(new_rate);
        }

        unsafe extern "C" fn ffi_push_audio_samples(
            context: *mut c_void,
            data: *const c_void,
            length: usize,
        ) {
            let context = context as *mut A;
            let data_ptr = data as *const u16;
            (*context).push_audio_samples(std::slice::from_raw_parts(data_ptr, length / 2));
        }
    }

    impl<A: AudioHandler> Drop for AudioHandlerFFI<A> {
        fn drop(&mut self) {
            mem::drop(unsafe { Box::from_raw(self.0) });
        }
    }

    pub(crate) struct SaveHandlerFFI<S: SaveHandler>(*mut S);

    unsafe impl<S: SaveHandler> Send for SaveHandlerFFI<S> {}
    impl<S: SaveHandler> FFIHandler for SaveHandlerFFI<S> {}

    impl<S: SaveHandler> SaveHandlerFFI<S> {
        pub fn new(handler: S) -> Self {
            let heap_alloc = Box::into_raw(Box::new(handler));
            Self(heap_alloc)
        }

        pub(super) unsafe fn create_ffi_handler(&self) -> m64prs_sys::TasSaveHandler {
            m64prs_sys::TasSaveHandler {
                context: self.0 as *mut c_void,
                signature: S::SIGNATURE,
                version: S::VERSION,
                alloc_size: S::ALLOC_SIZE,
                save_extra_data: Some(Self::ffi_save_extra_data),
                load_extra_data: Some(Self::ffi_load_extra_data),
                get_data_size: Some(Self::ffi_get_data_size),
            }
        }

        unsafe extern "C" fn ffi_save_extra_data(
            context: *mut c_void,
            data: *mut c_char,
            size: usize,
        ) {
            let context = context as *mut S;
            (*context).save_extra_data(std::slice::from_raw_parts_mut(data as *mut u8, size));
        }

        unsafe extern "C" fn ffi_load_extra_data(
            context: *mut c_void,
            version: u32,
            data: *const c_char,
            size: usize,
        ) {
            let context = context as *mut S;
            (*context)
                .load_extra_data(version, std::slice::from_raw_parts(data as *const u8, size));
        }

        unsafe extern "C" fn ffi_get_data_size(context: *mut c_void, version: u32) -> usize {
            let context = context as *mut S;
            (*context).get_data_size(version)
        }
    }

    impl<S: SaveHandler> Drop for SaveHandlerFFI<S> {
        fn drop(&mut self) {
            mem::drop(unsafe { Box::from_raw(self.0) });
        }
    }
}
