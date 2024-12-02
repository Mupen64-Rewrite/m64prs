use std::{
    env,
    error::Error,
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use m64prs_core::{
    error::{M64PError, SavestateError},
    plugin::PluginSet,
    save::SavestateFormat,
    Core,
};
use m64prs_sys::{CoreParam, EmuState};
use m64prs_vcr::VcrState;
use num_enum::TryFromPrimitive;
use relm4::{ComponentSender, Sender};

use crate::ui::core::{vidext::{VideoExtensionParameters, VideoExtensionState}, CoreRequest};

use super::{vidext::VidextResponse, CoreResponse, MupenCore};

#[derive(Debug)]
pub(super) enum CoreState {
    Uninit,
    Ready(CoreReadyState),
    Running(CoreRunningState),
}

#[derive(Debug)]
pub(super) struct CoreReadyState {
    core: Core,
}
#[derive(Debug)]
pub(super) struct CoreRunningState {
    core: Arc<Core>,
    join_handle: JoinHandle<Result<(), M64PError>>,
    vcr_read_only: bool,
    vcr_state: Arc<Mutex<Option<VcrState>>>,
}

pub(super) type VidextSender = mpsc::Sender<(usize, VidextResponse)>;

impl Default for CoreState {
    fn default() -> Self {
        Self::Uninit
    }
}

impl CoreState {
    pub(super) fn is_uninit(&self) -> bool {
        matches!(self, Self::Uninit)
    }

    pub(super) fn take(&mut self) -> CoreState {
        std::mem::take(self)
    }

    pub(super) fn borrow_ready(&mut self) -> Option<&mut CoreReadyState> {
        match self {
            CoreState::Ready(ready_state) => Some(ready_state),
            _ => None,
        }
    }

    pub(super) fn borrow_running(&mut self) -> Option<&mut CoreRunningState> {
        match self {
            CoreState::Running(running_state) => Some(running_state),
            _ => None,
        }
    }
}

impl CoreReadyState {
    pub(super) fn new(sender: &ComponentSender<MupenCore>) -> (Self, VidextSender) {
        #[cfg(target_os = "windows")]
        const MUPEN_FILENAME: &str = "mupen64plus.dll";
        #[cfg(target_os = "macos")]
        const MUPEN_FILENAME: &str = "libmupen64plus.dylib";
        #[cfg(target_os = "linux")]
        const MUPEN_FILENAME: &str = "libmupen64plus.so";

        let self_path = env::current_exe().expect("should be able to find current_exe");
        let self_dir = self_path
            .parent()
            .expect("self path should be in a directory");

        let mupen_dll_path = self_dir.join(MUPEN_FILENAME);
        let data_dir = self_dir.join("data\\");

        log::info!("Loading M64+ from {}", mupen_dll_path.display());
        log::info!("Data path is {}", data_dir.display());

        let mut core = m64prs_core::Core::init(mupen_dll_path, None, Some(&data_dir))
            .expect("core startup should succeed");

        let (vidext, vidext_inbound) = VideoExtensionParameters::new(sender.output_sender().clone());

        core.override_vidext::<VideoExtensionState, _>(Some(vidext))
            .expect("vidext override should succeed");

        {
            let sender = sender.clone();
            core.listen_state(move |param, value| match param {
                CoreParam::EmuState => {
                    sender.input(CoreRequest::EmuStateChanged(
                        (value as <EmuState as TryFromPrimitive>::Primitive)
                            .try_into()
                            .unwrap(),
                    ));
                }
                CoreParam::SavestateSlot => {
                    let _ = sender.output(CoreResponse::SavestateSlotChanged(value as u8));
                }
                _ => (),
            });
        }

        (Self { core }, vidext_inbound)
    }

    pub(super) fn start_rom(
        mut self,
        rom_data: &[u8],
        plugins: PluginSet,
        sender: &Sender<CoreResponse>
    ) -> Result<CoreRunningState, (Box<dyn Error + Send + Sync>, Self)> {
        if let Err(err) = self.core.open_rom(rom_data) {
            return Err((Box::new(err), self));
        }
        if let Err(err) = self.core.attach_plugins(plugins) {
            self.core.close_rom().unwrap();
            return Err((Box::new(err), self));
        }

        let core = Arc::new(self.core);
        let join_handle = {
            let core = core.clone();
            thread::spawn(move || core.execute())
        };

        let vcr_state = Arc::new(Mutex::new(None));
        let vcr_read_only = true;

        sender.emit(CoreResponse::VcrReadOnlyChanged(vcr_read_only));

        Ok(CoreRunningState {
            core,
            join_handle,
            vcr_read_only,
            vcr_state,
        })
    }
}

impl CoreRunningState {
    pub(super) fn stop_rom(self) -> (CoreReadyState, Option<M64PError>) {
        // stop the core
        let _ = self.core.request_stop();
        let error = self.join_handle.join().unwrap().err();

        // this should now be the only remaining reference; so extract the core
        let core = Arc::into_inner(self.core).expect("this should be the only ref to core");

        (CoreReadyState { core }, error)
    }

    pub(super) fn toggle_pause(&mut self) -> Result<(), M64PError> {
        match self.core.emu_state() {
            EmuState::Running => self.core.request_pause(),
            EmuState::Paused => self.core.request_resume(),
            _ => unreachable!(),
        }
    }

    pub(super) fn advance_frame(&mut self) -> Result<(), M64PError> {
        self.core.request_advance_frame()
    }

    pub(super) fn reset(&mut self, hard: bool) -> Result<(), M64PError> {
        self.core.reset(hard)
    }

    pub(super) async fn save_slot(&mut self) -> Result<(), SavestateError> {
        self.core.save_slot().await
    }

    pub(super) async fn load_slot(&mut self) -> Result<(), SavestateError> {
        self.core.load_slot().await
    }

    pub(super) fn set_save_slot(&mut self, slot: u8) -> Result<(), M64PError> {
        self.core.set_state_slot(slot)
    }

    pub(super) async fn save_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SavestateError> {
        self.core
            .save_file(path.as_ref(), SavestateFormat::Mupen64Plus)
            .await
    }

    pub(super) async fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SavestateError> {
        self.core.load_file(path.as_ref()).await
    }

    pub(super) fn toggle_read_only(&mut self, sender: &Sender<CoreResponse>) {
        self.vcr_read_only ^= true;
        sender.emit(CoreResponse::VcrReadOnlyChanged(self.vcr_read_only));
    }
}
