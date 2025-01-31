use std::{
    ffi::{c_void, CStr},
    mem,
};

use decan::can::NonOwningCan;
use m64prs_sys::{
    api::PluginCoreApi, common::M64PError, ptr_DebugCallback, DynlibHandle, MsgLevel,
};

pub mod config;
pub mod logging;

pub struct Core {
    debug_ctx: *mut c_void,
    debug_callback: ptr_DebugCallback,

    api: NonOwningCan<PluginCoreApi>,
}

unsafe impl Send for Core {}
unsafe impl Sync for Core {}

impl Core {
    /// Initializes a core handle for a plugin.
    pub unsafe fn new(
        core_handle: DynlibHandle,
        debug_ctx: *mut c_void,
        debug_callback: ptr_DebugCallback,
    ) -> Result<Self, M64PError> {
        let api = NonOwningCan::wrap_raw(mem::transmute::<_, decan::raw::Handle>(core_handle))
            .map_err(|_| M64PError::SystemFail)?;

        Ok(Self {
            debug_ctx,
            debug_callback,
            api,
        })
    }

    /// Logs a debug message to the frontend.
    pub fn debug_message(&self, level: MsgLevel, message: &CStr) {
        self.debug_callback.inspect(|callback| unsafe {
            callback(self.debug_ctx, level as i32, message.as_ptr());
        });
    }
}

/// Internal helper function to convert C results to Rust errors.
#[inline(always)]
fn core_fn(err: m64prs_sys::Error) -> Result<(), M64PError> {
    match err {
        m64prs_sys::Error::Success => Ok(()),
        err => Err(M64PError::try_from(err).unwrap()),
    }
}
