use std::{ffi::c_void, path::Path, process::{Child, Command}};

use m64prs_plugin_core::Core;
use m64prs_sys::{common::M64PError, key::{Mod, Scancode}, ptr_DebugCallback, ControlInfo, DynlibHandle};
use tasinput_protocol::{gen_socket_id, types::PortMask, Endpoint, HostMessage, HostReply, HostRequest, UiMessage, UiReply, UiRequest};

use crate::{util::{exe_name, ControlsRef}};

pub(crate) struct PluginState {
    _core: Core,
    ui_host: Child,
    endpoint: Endpoint<HostMessage, UiMessage>,
    controllers: Option<ControlsRef>,
}

impl PluginState {
    pub(crate) fn startup(
        lib_path: &Path,
        core_handle: DynlibHandle,
        debug_ctx: *mut c_void,
        debug_callback: ptr_DebugCallback,
    ) -> Result<Self, M64PError> {
        let core = unsafe {
            Core::new(core_handle, debug_ctx, debug_callback).map_err(|_| M64PError::SystemFail)?
        };

        let ui_host_path = lib_path
            .parent()
            .map(|dir| dir.join(exe_name("tasinput-ui")))
            .ok_or_else(|| {
                log::error!(
                    "Failed to find tasinput-ui. Ensure it is in the same directory as {}",
                    lib_path
                        .file_name()
                        .map_or("<unknown>".to_owned(), |name| name
                            .to_string_lossy()
                            .to_string())
                );
                M64PError::Internal
            })?;

        let socket_id = gen_socket_id("tasinput-");

        let endpoint = Endpoint::listen(&socket_id, |request| async {
            match request {
                UiRequest::Dummy => HostReply::Ack,
            }
        }).map_err(|_| {
            log::error!("Failed to setup IPC runtime!");
            M64PError::SystemFail
        })?;

        let ui_host = Command::new(&ui_host_path)
            .args(["--server-socket", &socket_id])
            .spawn()
            .map_err(|_| {
                log::error!("Unabled to start tasinput-ui!");
                M64PError::Internal
            })?;

        let endpoint = endpoint.ready_blocking().map_err(|_| {
            log::error!("Failed to setup IPC runtime! (listen)");
            M64PError::SystemFail
        })?;

        Ok(Self {
            _core: core,
            ui_host,
            endpoint,
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

    pub(crate) fn key_down(&mut self, sdl_key: Scancode, sdl_mod: Mod) {
        if sdl_key == Scancode::P {
            println!("ping");
            let _ = self.endpoint.post_request_blocking(HostRequest::Ping).blocking_recv();
            println!("pong");
        }
    }
}

impl Drop for PluginState {
    fn drop(&mut self) {
        let _ = self.ui_host.kill();
    }
}