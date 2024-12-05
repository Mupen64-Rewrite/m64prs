use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
};

use m64prs_core::{
    error::{M64PError, SavestateError},
    plugin::PluginSet,
    tas_callbacks::SaveHandler,
    Plugin,
};
use m64prs_sys::EmuState;
use pollster::FutureExt;
use relm4::{Component, ComponentParts, ComponentSender};
use state::{CoreReadyState, CoreState};
use vidext::VidextResponse;

mod state;
pub mod vidext;

#[derive(Default)]
struct TestSaveHandler {
    counter: u64,
}

impl SaveHandler for TestSaveHandler {
    const SIGNATURE: u32 = 0xDEADBEEF;

    fn save_xd(&mut self) -> Result<Box<[u8]>, Box<dyn Error>> {
        log::debug!("Saved XD: {}", self.counter);
        let result = Box::new(self.counter.to_le_bytes());
        self.counter += 1;
        Ok(result)
    }

    fn load_xd(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let value = u64::from_le_bytes(<[u8; 8]>::try_from(&data[0..8]).unwrap());
        log::debug!("Loaded XD: {}", value);
        Ok(())
    }
}

#[derive(Debug)]
pub enum CoreRequest {
    Init,
    EmuStateChanged(EmuState),

    StartRom(PathBuf),
    StopRom,

    TogglePause,
    FrameAdvance,
    Reset(bool),

    SaveSlot,
    LoadSlot,
    SetSaveSlot(u8),
    SaveFile(PathBuf),
    LoadFile(PathBuf),

    ToggleVcrReadOnly,
}

#[derive(Debug)]
pub enum CoreResponse {
    CoreReady {
        vidext_inbound: mpsc::Sender<(usize, VidextResponse)>,
    },
    Error(Box<dyn Error + Send + Sync>),
    EmuStateChanged(EmuState),
    SavestateSlotChanged(u8),
    StateRequestStarted,
    StateRequestComplete,
    VcrReadOnlyChanged(bool),
    VidextRequest(usize, vidext::VidextRequest),
}

#[derive(Debug)]
pub struct MupenCore(CoreState);

impl MupenCore {
    fn init(&mut self, sender: &ComponentSender<Self>) {
        if !self.0.is_uninit() {
            panic!("CoreRequest::Init should not be called externally");
        }

        let (ready_state, vidext_inbound) = CoreReadyState::new(sender);
        sender
            .output(CoreResponse::CoreReady { vidext_inbound })
            .expect("GUI should be able to catch this event!");
        self.0 = CoreState::Ready(ready_state);
    }

    fn state_change(
        &mut self,
        emu_state: EmuState,
        sender: &ComponentSender<Self>,
    ) -> Result<(), M64PError> {
        let _ = sender.output(CoreResponse::EmuStateChanged(emu_state));
        match (emu_state, self.0.take()) {
            (EmuState::Stopped, CoreState::Running(running_state)) => {
                let (ready_state, error) = running_state.stop_rom();
                self.0 = CoreState::Ready(ready_state);
                match error {
                    Some(err) => Err(err),
                    None => Ok(()),
                }
            }
            (_, inner) => {
                self.0 = inner;
                Ok(())
            }
        }
    }

    fn start_rom(
        &mut self,
        path: &Path,
        sender: &ComponentSender<Self>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let rom_data = fs::read(path)?;

        let plugins = {
            let self_path = std::env::current_exe().unwrap();
            let plugin_path = self_path.parent().unwrap().join("plugins");

            fn plugin_name(name: &str) -> String {
                #[cfg(target_os = "windows")]
                {
                    format!("{}.dll", name)
                }
                #[cfg(target_os = "macos")]
                {
                    format!("{}.dylib", name)
                }
                #[cfg(target_os = "linux")]
                {
                    format!("{}.so", name)
                }
            }

            // TODO: allow user to configure plugins
            PluginSet {
                graphics: Plugin::load(plugin_path.join(plugin_name("mupen64plus-video-rice")))?,
                audio: Plugin::load(plugin_path.join(plugin_name("mupen64plus-audio-sdl")))?,
                input: Plugin::load(plugin_path.join(plugin_name("mupen64plus-input-sdl")))?,
                rsp: Plugin::load(plugin_path.join(plugin_name("mupen64plus-rsp-hle")))?,
            }
        };

        if let CoreState::Ready(ready_state) = self.0.take() {
            match ready_state.start_rom(&rom_data, plugins, sender.output_sender()) {
                Ok(running_state) => {
                    self.0 = CoreState::Running(running_state);
                    Ok(())
                }
                Err((error, ready_state)) => {
                    self.0 = CoreState::Ready(ready_state);
                    Err(error)
                }
            }
        } else {
            panic!("expected CoreState::Ready for start_rom");
        }
    }

    fn stop_rom(&mut self) -> Result<(), M64PError> {
        match self.0.take() {
            CoreState::Running(running_state) => {
                let (ready_state, error) = running_state.stop_rom();
                self.0 = CoreState::Ready(ready_state);
                match error {
                    Some(err) => Err(err),
                    None => Ok(()),
                }
            }
            _ => panic!("expected CoreState::Running for stop_rom"),
        }
    }

    fn toggle_pause(&mut self) -> Result<(), M64PError> {
        self.0
            .borrow_running()
            .expect("expected CoreState::Running for toggle_pause")
            .toggle_pause()
    }

    fn advance_frame(&mut self) -> Result<(), M64PError> {
        self.0
            .borrow_running()
            .expect("expected CoreState::Running for advance_frame")
            .advance_frame()
    }

    fn reset(&mut self, hard: bool) -> Result<(), M64PError> {
        self.0
            .borrow_running()
            .expect("expected CoreState::Running for toggle_pause")
            .reset(hard)
    }

    fn save_slot(&mut self, sender: &ComponentSender<Self>) -> Result<(), SavestateError> {
        let running_state = self
            .0
            .borrow_running()
            .expect("expected CoreState::Running for save_slot");

        let _ = sender.output(CoreResponse::StateRequestStarted);
        let result = running_state.save_slot().block_on();
        let _ = sender.output(CoreResponse::StateRequestComplete);

        result
    }

    fn load_slot(&mut self, sender: &ComponentSender<Self>) -> Result<(), SavestateError> {
        let running_state = self
            .0
            .borrow_running()
            .expect("expected CoreState::Running for save_slot");

        let _ = sender.output(CoreResponse::StateRequestStarted);
        let result = running_state.load_slot().block_on();
        let _ = sender.output(CoreResponse::StateRequestComplete);

        result
    }

    fn save_file(
        &mut self,
        path: PathBuf,
        sender: &ComponentSender<Self>,
    ) -> Result<(), SavestateError> {
        let running_state = self
            .0
            .borrow_running()
            .expect("expected CoreState::Running for save_slot");

        let _ = sender.output(CoreResponse::StateRequestStarted);
        let result = running_state.save_file(path).block_on();
        let _ = sender.output(CoreResponse::StateRequestComplete);

        result
    }

    fn load_file(
        &mut self,
        path: PathBuf,
        sender: &ComponentSender<Self>,
    ) -> Result<(), SavestateError> {
        let running_state = self
            .0
            .borrow_running()
            .expect("expected CoreState::Running for save_slot");

        let _ = sender.output(CoreResponse::StateRequestStarted);
        let result = running_state.load_file(path).block_on();
        let _ = sender.output(CoreResponse::StateRequestComplete);

        result
    }

    fn set_save_slot(&mut self, slot: u8, sender: &ComponentSender<Self>) -> Result<(), M64PError> {
        self.0
            .borrow_running()
            .expect("expected CoreState::Running for toggle_pause")
            .set_save_slot(slot)
    }

    fn toggle_vcr_read_only(&mut self, sender: &ComponentSender<Self>) {
        self.0
            .borrow_running()
            .expect("expected CoreState::Running for toggle_pause")
            .toggle_read_only(sender.output_sender());
    }
}

impl Component for MupenCore {
    type Init = ();

    type Input = CoreRequest;
    type Output = CoreResponse;

    type CommandOutput = ();

    type Root = ();
    type Widgets = ();

    fn init(_init: (), _root: (), sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self(CoreState::default());
        sender.input(CoreRequest::Init);
        ComponentParts { model, widgets: () }
    }

    fn init_root() -> Self::Root {
        ()
    }

    fn update(&mut self, request: Self::Input, sender: ComponentSender<Self>, _root: &()) {
        use CoreRequest::*;
        let result: Result<(), Box<dyn Error + Send + Sync>> = match request {
            Init => Ok(self.init(&sender)),
            EmuStateChanged(emu_state) => self.state_change(emu_state, &sender).map_err(Into::into),
            StartRom(path) => self.start_rom(&path, &sender),
            StopRom => self.stop_rom().map_err(Into::into),
            TogglePause => self.toggle_pause().map_err(Into::into),
            FrameAdvance => self.advance_frame().map_err(Into::into),
            Reset(hard) => self.reset(hard).map_err(Into::into),
            SaveSlot => self.save_slot(&sender).map_err(Into::into),
            LoadSlot => self.load_slot(&sender).map_err(Into::into),
            SetSaveSlot(slot) => self.set_save_slot(slot, &sender).map_err(Into::into),
            SaveFile(path) => self.save_file(path, &sender).map_err(Into::into),
            LoadFile(path) => self.load_file(path, &sender).map_err(Into::into),
            ToggleVcrReadOnly => Ok(self.toggle_vcr_read_only(&sender)),
        };
        if let Err(error) = result {
            let _ = sender.output(CoreResponse::Error(error));
        }
    }
}
