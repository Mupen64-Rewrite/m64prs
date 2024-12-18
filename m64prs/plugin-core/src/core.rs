use std::{ffi::c_void, mem};

use decan::can::NonOwningCan;
use m64prs_sys::{api::PluginCoreApi, common::M64PError, ptr_DebugCallback, DynlibHandle};

mod config;

pub struct Core {
    debug_ctx: *mut c_void,
    debug_callback: ptr_DebugCallback,

    api: NonOwningCan<PluginCoreApi>,
}

impl Core {
    /// Initializes a core handle for a plugin.
    pub unsafe fn new(
        core_handle: DynlibHandle,
        debug_ctx: *mut c_void,
        debug_callback: ptr_DebugCallback,
    ) -> Result<Self, M64PError> {
        let api = NonOwningCan::wrap_raw(mem::transmute::<_, decan::raw::Handle>(core_handle))
            .map_err(|err| M64PError::SystemFail)?;

        Ok(Self { debug_ctx, debug_callback, api })
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