use std::{
    any::Any,
    env,
    error::Error,
    fs, mem,
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
};

use m64prs_core::{plugin::PluginSet, Plugin};
use m64prs_sys::{CoreParam, EmuState};
use num_enum::TryFromPrimitive;
use relm4::{ComponentSender, Worker};
use vidext::{VideoExtensionParameters, VideoExtensionState, VidextResponse};

pub mod vidext;

#[derive(Debug)]
pub enum Request {
    Init,

    StartRom(PathBuf),
    StopRom,
    TogglePause,
    FrameAdvance,
    Reset,

    CoreEmuStateChange(EmuState),
}

#[derive(Debug)]
pub enum Response {
    CoreReady {
        vidext_inbound: mpsc::Sender<(usize, VidextResponse)>,
    },
    Error(Box<dyn Error + Send + 'static>),
    EmuStateChange(EmuState),
    VidextRequest(usize, vidext::VidextRequest),
}

/// Inner enum representing the model's current state.
#[derive(Debug)]
enum ModelInner {
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
pub struct Model(ModelInner);

impl Model {
    fn init(&mut self, sender: &ComponentSender<Self>) {
        #[cfg(target_os = "windows")]
        const MUPEN_FILENAME: &str = "mupen64plus.dll";
        #[cfg(target_os = "macos")]
        const MUPEN_FILENAME: &str = "libmupen64plus.dylib";
        #[cfg(target_os = "linux")]
        const MUPEN_FILENAME: &str = "libmupen64plus.so";

        let vidext_inbound: mpsc::Sender<(usize, VidextResponse)>;

        self.0 = match self.0 {
            ModelInner::Uninit => {
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

                let param_box: Box<dyn Any> = Box::new(Some(vidext));

                core.override_vidext::<VideoExtensionState>(param_box)
                    .expect("vidext override should succeed");

                {
                    let sender = sender.clone();
                    core.listen_state(move |param, value| {
                        if let CoreParam::EmuState = param {
                            sender.input(Request::CoreEmuStateChange(
                                (value as <EmuState as TryFromPrimitive>::Primitive)
                                    .try_into()
                                    .unwrap(),
                            ));
                        }
                    });
                }

                ModelInner::Ready { core }
            }
            _ => panic!("core is already initialized"),
        };
        sender
            .output(Response::CoreReady { vidext_inbound })
            .unwrap();
    }

    fn start_rom(&mut self, path: &Path, sender: &ComponentSender<Self>) {
        self.0 = match mem::replace(&mut self.0, ModelInner::Uninit) {
            ModelInner::Uninit => panic!("core should be initialized"),
            ModelInner::Ready { core } => 'core_ready: {
                let rom_data = match fs::read(path) {
                    Ok(data) => data,
                    Err(error) => {
                        let _ = sender.output(Response::Error(Box::new(error)));
                        break 'core_ready ModelInner::Ready { core };
                    }
                };
                Self::start_rom_inner(&rom_data, core, sender)
            }
            ModelInner::Running {
                join_handle,
                core_ref,
            } => 'core_running: {
                let rom_data = match fs::read(path) {
                    Ok(data) => data,
                    Err(error) => {
                        let _ = sender.output(Response::Error(Box::new(error)));
                        break 'core_running ModelInner::Running {
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
        self.0 = match mem::replace(&mut self.0, ModelInner::Uninit) {
            ModelInner::Running {
                join_handle,
                core_ref,
            } => {
                let mut core = Self::stop_rom_inner(join_handle, core_ref, sender);

                core.detach_plugins();
                core.close_rom().expect("there should be an open ROM");

                ModelInner::Ready { core }
            }
            _ => panic!("core should be running"),
        };
    }

    fn state_change(&mut self, emu_state: EmuState, sender: &ComponentSender<Self>) {
        match (emu_state, mem::replace(&mut self.0, ModelInner::Uninit)) {
            // The core has stopped on its own since we last started it
            (
                EmuState::Stopped,
                ModelInner::Running {
                    join_handle,
                    core_ref,
                },
            ) => {
                join_handle.join().expect("the core thread shouldn't panic");

                let mut core = Arc::into_inner(core_ref)
                    .expect("no refs to the core should exist outside of the emulator thread");

                core.detach_plugins();
                core.close_rom().expect("there should be an open ROM");

                self.0 = ModelInner::Ready { core };
            }
            // Nothing interesting
            (_, inner) => self.0 = inner,
        }

        // Forward state change to frontend
        let _ = sender.output(Response::EmuStateChange(emu_state));
    }

    fn toggle_pause(&mut self, _sender: &ComponentSender<Self>) {
        match &mut self.0 {
            ModelInner::Running {
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
            ModelInner::Running {
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
            ModelInner::Running {
                join_handle: _,
                core_ref,
            } => core_ref
                .reset(false)
                .expect("command execution should not fail"),
            _ => panic!("core should be running"),
        }
    }
}

/// Internal functions behind the requests.
impl Model {
    fn start_rom_inner(
        rom_data: &[u8],
        mut core: m64prs_core::Core,
        sender: &ComponentSender<Self>,
    ) -> ModelInner {
        macro_rules! check {
            ($res:expr) => {
                match ($res) {
                    Ok(value) => value,
                    Err(err) => {
                        let _ = sender.output(Response::Error(Box::new(err)));
                        return ModelInner::Ready { core };
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
            let _ = sender.output(Response::Error(Box::new(err)));
            core.close_rom().expect("there should be an open ROM");
            return ModelInner::Ready { core };
        }

        let core_ref = Arc::new(core);

        let join_handle = {
            let core = Arc::clone(&core_ref);
            thread::spawn(move || {
                let _ = core.execute();
            })
        };

        ModelInner::Running {
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

impl Worker for Model {
    type Init = ();

    type Input = Request;
    type Output = Response;

    fn init(_: Self::Init, sender: ComponentSender<Self>) -> Self {
        let result = Self(ModelInner::Uninit);
        sender.input(Request::Init);
        result
    }

    fn update(&mut self, request: Self::Input, sender: ComponentSender<Self>) {
        match request {
            Request::Init => self.init(&sender),
            Request::StartRom(path) => self.start_rom(&path, &sender),
            Request::StopRom => self.stop_rom(&sender),
            Request::CoreEmuStateChange(emu_state) => self.state_change(emu_state, &sender),
            Request::TogglePause => self.toggle_pause(&sender),
            Request::FrameAdvance => self.advance_frame(&sender),
            Request::Reset => self.reset(&sender),
        }
    }
}
