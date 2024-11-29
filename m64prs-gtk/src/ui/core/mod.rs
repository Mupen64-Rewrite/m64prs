use std::{
    any::Any,
    env,
    error::Error,
    fs, mem,
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
};

use m64prs_core::{error::SavestateError, plugin::PluginSet, save::SavestateFormat, Plugin};
use m64prs_sys::{CoreParam, EmuState};
use num_enum::TryFromPrimitive;
use relm4::{Component, ComponentParts, ComponentSender, Worker};
use vidext::{VideoExtensionParameters, VideoExtensionState, VidextResponse};

pub mod vidext;

#[derive(Debug)]
pub enum MupenCoreRequest {
    Init,
    CoreEmuStateChange(EmuState),

    StartRom(PathBuf),
    StopRom,

    TogglePause,
    FrameAdvance,
    Reset,

    SaveSlot,
    LoadSlot,
    SetSaveSlot(u8),
    SaveFile(PathBuf),
    LoadFile(PathBuf),
}

#[derive(Debug)]
pub enum MupenCoreResponse {
    CoreReady {
        vidext_inbound: mpsc::Sender<(usize, VidextResponse)>,
    },
    Error(Box<dyn Error + Send + 'static>),
    EmuStateChanged(EmuState),
    SavestateSlotChanged(u8),
    StateRequestStarted,
    StateRequestComplete,
    VidextRequest(usize, vidext::VidextRequest),
}

#[derive(Debug)]
pub enum MupenCoreCommandResponse {
    LoadComplete(Result<(), SavestateError>),
    SaveComplete(Result<(), SavestateError>),
}

/// Inner enum representing the model's current state.
#[derive(Debug)]
enum MupenCoreInner {
    /// The core is not initialized yet.
    Uninit,
    /// The core is ready to use. It does not have a ROM open.
    Ready { core: m64prs_core::Core },
    /// The core is running a ROM in a background thread.
    Running {
        join_handle: JoinHandle<()>,
        core_ref: Arc<m64prs_core::Core>,
    },
}

#[derive(Debug)]
pub struct MupenCore(MupenCoreInner);

impl MupenCore {
    fn init(&mut self, sender: &ComponentSender<Self>) {
        #[cfg(target_os = "windows")]
        const MUPEN_FILENAME: &str = "mupen64plus.dll";
        #[cfg(target_os = "macos")]
        const MUPEN_FILENAME: &str = "libmupen64plus.dylib";
        #[cfg(target_os = "linux")]
        const MUPEN_FILENAME: &str = "libmupen64plus.so";

        let vidext_inbound: mpsc::Sender<(usize, VidextResponse)>;

        self.0 = match self.0 {
            MupenCoreInner::Uninit => {
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

                let vidext: VideoExtensionParameters;
                (vidext, vidext_inbound) = VideoExtensionParameters::new(sender.clone());

                core.override_vidext::<VideoExtensionState, _>(Some(vidext))
                    .expect("vidext override should succeed");

                // let save_handler = TestSaveHandler::default();
                // core.set_save_handler(save_handler)
                //     .expect("save handler override should succeed");

                {
                    let sender = sender.clone();
                    core.listen_state(move |param, value| match param {
                        CoreParam::EmuState => {
                            sender.input(MupenCoreRequest::CoreEmuStateChange(
                                (value as <EmuState as TryFromPrimitive>::Primitive)
                                    .try_into()
                                    .unwrap(),
                            ));
                        }
                        CoreParam::SavestateSlot => {
                            let _ =
                                sender.output(MupenCoreResponse::SavestateSlotChanged(value as u8));
                        }
                        _ => (),
                    });
                }

                MupenCoreInner::Ready { core }
            }
            _ => panic!("core is already initialized"),
        };
        sender
            .output(MupenCoreResponse::CoreReady { vidext_inbound })
            .unwrap();
    }

    fn start_rom(&mut self, path: &Path, sender: &ComponentSender<Self>) {
        self.0 = match mem::replace(&mut self.0, MupenCoreInner::Uninit) {
            MupenCoreInner::Uninit => panic!("core should be initialized"),
            MupenCoreInner::Ready { core } => 'core_ready: {
                let rom_data = match fs::read(path) {
                    Ok(data) => data,
                    Err(error) => {
                        let _ = sender.output(MupenCoreResponse::Error(Box::new(error)));
                        break 'core_ready MupenCoreInner::Ready { core };
                    }
                };
                Self::start_rom_inner(&rom_data, core, sender)
            }
            MupenCoreInner::Running {
                join_handle,
                core_ref,
            } => 'core_running: {
                let rom_data = match fs::read(path) {
                    Ok(data) => data,
                    Err(error) => {
                        let _ = sender.output(MupenCoreResponse::Error(Box::new(error)));
                        break 'core_running MupenCoreInner::Running {
                            join_handle,
                            core_ref,
                        };
                    }
                };
                let core = Self::stop_rom_inner(join_handle, core_ref, sender);
                Self::start_rom_inner(&rom_data, core, sender)
            }
        };
    }

    fn stop_rom(&mut self, sender: &ComponentSender<Self>) {
        self.0 = match mem::replace(&mut self.0, MupenCoreInner::Uninit) {
            MupenCoreInner::Running {
                join_handle,
                core_ref,
            } => {
                let mut core = Self::stop_rom_inner(join_handle, core_ref, sender);

                core.detach_plugins();
                core.close_rom().expect("there should be an open ROM");

                MupenCoreInner::Ready { core }
            }
            _ => panic!("core should be running"),
        };
    }

    fn toggle_pause(&mut self, _sender: &ComponentSender<Self>) {
        match &mut self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => match core_ref.emu_state() {
                EmuState::Running => core_ref.request_pause(),
                EmuState::Paused => core_ref.request_resume(),
                _ => unreachable!(),
            }
            .expect("command execution should not fail"),
            _ => panic!("core should be running"),
        };
    }

    fn advance_frame(&mut self, _sender: &ComponentSender<Self>) {
        match &mut self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => core_ref
                .request_advance_frame()
                .expect("command execution should not fail"),
            _ => panic!("core should be running"),
        }
    }

    fn reset(&mut self, _sender: &ComponentSender<Self>) {
        match &mut self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => core_ref
                .reset(false)
                .expect("command execution should not fail"),
            _ => panic!("core should be running"),
        }
    }

    fn state_change(&mut self, emu_state: EmuState, sender: &ComponentSender<Self>) {
        match (emu_state, mem::replace(&mut self.0, MupenCoreInner::Uninit)) {
            // The core has stopped on its own since we last started it
            (
                EmuState::Stopped,
                MupenCoreInner::Running {
                    join_handle,
                    core_ref,
                },
            ) => {
                join_handle.join().expect("the core thread shouldn't panic");

                let mut core = Arc::into_inner(core_ref)
                    .expect("no refs to the core should exist outside of the emulator thread");

                core.detach_plugins();
                core.close_rom().expect("there should be an open ROM");

                self.0 = MupenCoreInner::Ready { core };
            }
            // Nothing interesting
            (_, inner) => self.0 = inner,
        }

        // Forward state change to frontend
        let _ = sender.output(MupenCoreResponse::EmuStateChanged(emu_state));
    }

    fn save_slot(&mut self, sender: &ComponentSender<Self>) {
        let core_ref = match &self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => Arc::clone(core_ref),
            _ => panic!("core should be running"),
        };

        let _ = sender.output(MupenCoreResponse::StateRequestStarted);

        sender.oneshot_command(async move {
            let result = core_ref.save_state().await;
            MupenCoreCommandResponse::SaveComplete(result)
        });
    }

    fn load_slot(&mut self, sender: &ComponentSender<Self>) {
        let core_ref = match &self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => Arc::clone(core_ref),
            _ => panic!("core should be running"),
        };

        sender.oneshot_command(async move {
            let result = core_ref.load_state().await;
            MupenCoreCommandResponse::LoadComplete(result)
        });
    }

    fn save_file(&mut self, path: PathBuf, sender: &ComponentSender<Self>) {
        let core_ref = match &self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => Arc::clone(core_ref),
            _ => panic!("core should be running"),
        };

        let _ = sender.output(MupenCoreResponse::StateRequestStarted);

        sender.oneshot_command(async move {
            let result = core_ref.save_file(path, SavestateFormat::Mupen64Plus).await;
            MupenCoreCommandResponse::SaveComplete(result)
        });
    }

    fn load_file(&mut self, path: PathBuf, sender: &ComponentSender<Self>) {
        let core_ref = match &self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => Arc::clone(core_ref),
            _ => panic!("core should be running"),
        };

        let _ = sender.output(MupenCoreResponse::StateRequestStarted);

        sender.oneshot_command(async move {
            let result = core_ref.load_file(path).await;
            MupenCoreCommandResponse::SaveComplete(result)
        });
    }

    fn save_op_complete(
        &mut self,
        result: Result<(), SavestateError>,
        sender: &ComponentSender<Self>,
    ) {
        let _ = sender.output(MupenCoreResponse::StateRequestComplete);
        if let Err(error) = result {
            let _ = sender.output(MupenCoreResponse::Error(Box::new(error)));
        }
    }

    fn set_save_slot(&mut self, slot: u8, _sender: &ComponentSender<Self>) {
        match &mut self.0 {
            MupenCoreInner::Running {
                join_handle: _,
                core_ref,
            } => core_ref
                .set_savestate_slot(slot)
                .expect("savestate slot set should succeed"),
            _ => panic!("core should be running"),
        }
    }
}

/// Internal functions behind the requests.
impl MupenCore {
    fn start_rom_inner(
        rom_data: &[u8],
        mut core: m64prs_core::Core,
        sender: &ComponentSender<Self>,
    ) -> MupenCoreInner {
        macro_rules! check {
            ($res:expr) => {
                match ($res) {
                    Ok(value) => value,
                    Err(err) => {
                        let _ = sender.output(MupenCoreResponse::Error(Box::new(err)));
                        return MupenCoreInner::Ready { core };
                    }
                }
            };
        }

        let self_path = env::current_exe().expect("should be able to find current_exe");
        let self_dir = self_path
            .parent()
            .expect("self path should be in a directory");

        #[cfg(target_os = "windows")]
        const DLL_EXT: &str = "dll";
        #[cfg(target_os = "macos")]
        const DLL_EXT: &str = "dylib";
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        const DLL_EXT: &str = "so";

        let plugins = PluginSet {
            graphics: check!(Plugin::load(
                self_dir.join(format!("plugins/mupen64plus-video-rice.{}", DLL_EXT))
            )),
            audio: check!(Plugin::load(
                self_dir.join(format!("plugins/mupen64plus-audio-sdl.{}", DLL_EXT))
            )),
            input: check!(Plugin::load(
                self_dir.join(format!("plugins/mupen64plus-input-sdl.{}", DLL_EXT))
            )),
            rsp: check!(Plugin::load(
                self_dir.join(format!("plugins/mupen64plus-rsp-hle.{}", DLL_EXT))
            )),
        };

        check!(core.open_rom(rom_data));

        if let Err(err) = core.attach_plugins(plugins) {
            let _ = sender.output(MupenCoreResponse::Error(Box::new(err)));
            core.close_rom().expect("there should be an open ROM");
            return MupenCoreInner::Ready { core };
        }

        let core_ref = Arc::new(core);

        let join_handle = {
            let core = Arc::clone(&core_ref);
            thread::spawn(move || {
                let _ = core.execute();
            })
        };

        MupenCoreInner::Running {
            join_handle,
            core_ref,
        }
    }

    fn stop_rom_inner(
        join_handle: JoinHandle<()>,
        core_ref: Arc<m64prs_core::Core>,
        _sender: &ComponentSender<Self>,
    ) -> m64prs_core::Core {
        core_ref.request_stop().expect("Core::stop should succeed");
        join_handle.join().expect("the core thread shouldn't panic");

        Arc::into_inner(core_ref)
            .expect("no refs to the core should exist outside of the emulator thread")
    }
}

impl Component for MupenCore {
    type Init = ();

    type Input = MupenCoreRequest;
    type Output = MupenCoreResponse;

    type CommandOutput = MupenCoreCommandResponse;

    type Root = ();
    type Widgets = ();

    fn init(_init: (), _root: (), sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self(MupenCoreInner::Uninit);
        sender.input(MupenCoreRequest::Init);
        ComponentParts { model, widgets: () }
    }

    fn init_root() -> Self::Root {
        ()
    }

    fn update(&mut self, request: Self::Input, sender: ComponentSender<Self>, _root: &()) {
        use MupenCoreRequest::*;
        match request {
            Init => self.init(&sender),
            CoreEmuStateChange(emu_state) => self.state_change(emu_state, &sender),
            StartRom(path) => self.start_rom(&path, &sender),
            StopRom => self.stop_rom(&sender),
            TogglePause => self.toggle_pause(&sender),
            FrameAdvance => self.advance_frame(&sender),
            Reset => self.reset(&sender),
            SaveSlot => self.save_slot(&sender),
            LoadSlot => self.load_slot(&sender),
            SetSaveSlot(slot) => self.set_save_slot(slot, &sender),
            SaveFile(path) => self.save_file(path, &sender),
            LoadFile(path) => self.load_file(path, &sender),
        }
    }

    fn update_cmd(
        &mut self,
        response: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match response {
            MupenCoreCommandResponse::LoadComplete(result) => {
                self.save_op_complete(result, &sender)
            }
            MupenCoreCommandResponse::SaveComplete(result) => {
                self.save_op_complete(result, &sender)
            }
        }
    }
}
