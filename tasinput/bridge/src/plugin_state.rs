use std::{
    ffi::c_void,
    path::Path,
    process::{Child, Command},
    time::Duration,
};

use m64prs_plugin_core::Core;
use m64prs_sys::{common::M64PError, ptr_DebugCallback, Buttons, ControlInfo, DynlibHandle};
use tasinput_protocol::{
    gen_socket_id, types::PortMask, Endpoint, HostMessage, HostReply, HostRequest, UiMessage,
    UiReply, UiRequest, PING_INTERVAL,
};
use tokio::time::{interval, timeout, MissedTickBehavior};
use wait_timeout::ChildExt;

use crate::util::{exe_name, ControlsRef};

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
        })
        .map_err(|_| {
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

        let mut endpoint = endpoint.ready_blocking().map_err(|_| {
            log::error!("Failed to setup IPC runtime! (listen)");
            M64PError::SystemFail
        })?;

        // setup future to regularly ping the client
        endpoint.spawn(|mut handle| async move {
            const PING_ROUNDTRIP_TIMEOUT: Duration = Duration::from_millis(500);

            let mut timer = interval(PING_INTERVAL);
            timer.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let cancel = handle.cancel_token();

            loop {
                tokio::select! {
                    biased;
                    _ = cancel.cancelled() => {
                        return;
                    }
                    _ = timer.tick() => {
                        match timeout(
                            PING_ROUNDTRIP_TIMEOUT,
                            handle.post_request(HostRequest::Ping).await
                        )
                        .await {
                            Ok(_) => (),
                            Err(timeout) => {
                                // ping timed out
                                log::warn!("tasinput-ui ping timed out after {}", timeout);
                            },
                        }
                    }
                }
            }
        });

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
        self.endpoint
            .post_request_blocking(HostRequest::InitControllers {
                active: PortMask::PORT1,
            })
            .unwrap();

        self.controllers = Some(controllers);
    }

    pub(crate) fn get_keys(&mut self, controller: u8) -> Buttons {
        let reply = self
            .endpoint
            .post_request_blocking(HostRequest::PollState { controller })
            .unwrap();
        match reply {
            UiReply::PolledState { buttons } => buttons,
            _ => panic!(),
        }
    }

    pub(crate) fn rom_open(&mut self) {
        self.endpoint
            .post_request_blocking(HostRequest::SetVisible { visible: true })
            .unwrap();
    }

    pub(crate) fn rom_closed(&mut self) {
        self.endpoint
            .post_request_blocking(HostRequest::SetVisible { visible: false })
            .unwrap();
    }
}

impl Drop for PluginState {
    fn drop(&mut self) {
        let _ = self
            .endpoint
            .post_request_blocking(HostRequest::Close);
        match self
            .ui_host
            .wait_timeout(Duration::from_millis(250))
            .unwrap()
        {
            Some(exit_code) => {
                log::info!("tasinput-ui exited (code {})", exit_code);
            }
            None => {
                log::info!("killing tasinput-ui");
                let _ = self.ui_host.kill();
            }
        }
    }
}
