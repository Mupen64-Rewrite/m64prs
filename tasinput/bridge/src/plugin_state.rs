use std::{cell::UnsafeCell, ffi::c_void};

use m64prs_plugin_core::Core;
use m64prs_sys::{common::M64PError, ptr_DebugCallback, Control, ControlInfo, DynlibHandle};

use crate::util::ControlsRef;

pub(crate) struct PluginState {
    core: Core,
    controllers: Option<ControlsRef>,
}

impl PluginState {
    pub(crate) fn init(
        core_handle: DynlibHandle,
        debug_ctx: *mut c_void,
        debug_callback: ptr_DebugCallback,
    ) -> Result<Self, M64PError> {
        let core = unsafe {
            Core::new(core_handle, debug_ctx, debug_callback).map_err(|_| M64PError::SystemFail)?
        };

        Ok(Self {
            core,
            controllers: None,
        })
    }

    pub(crate) fn init_controllers(&mut self, info: ControlInfo) {
        let mut controllers = ControlsRef::new(info.Controls);

        unsafe {
            controllers.index_mut(0).Present = 1;
        }

        self.controllers = Some(controllers);
    }
}
