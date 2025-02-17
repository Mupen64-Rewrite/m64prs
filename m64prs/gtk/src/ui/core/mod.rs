use std::{
    borrow::Borrow, cell::Cell, error::Error, ffi::CStr, fs, path::{Path, PathBuf}, sync::Arc
};

use futures::{executor::block_on, lock::Mutex};
use gdk::prelude::SurfaceExt;
use glib::SendWeakRef;
use gtk::prelude::NativeExt;
use m64prs_core::{
    config::ConfigSection,
    error::{M64PError, PluginLoadError, SavestateError},
    plugin::{PluginInfo, PluginSet, PluginType},
    save::SavestateFormat,
    tas_callbacks::{FrameHandler, InputHandler, SaveHandler},
    ConfigSectionMut, Core,
};
use m64prs_sys::{CoreParam, EmuState, RomHeader, RomSettings};
use m64prs_vcr::{movie::M64File, VcrState};
use num_enum::TryFromPrimitive;
use threading::RunningCore;
use vidext::{VideoExtensionParameters, VideoExtensionState};

use crate::utils::{
    keyboard,
    paths::{CONFIG_DIR, INSTALL_DIRS},
};

use super::{main_window::MainWindow, settings_dialog};

mod threading;
mod vidext;

#[derive(Debug)]
pub enum CoreState {
    Uninit,
    Ready(CoreReadyState),
    Running(CoreRunningState),
}

#[derive(Debug)]
pub struct CoreReadyState {
    core: Core,
    main_window_ref: SendWeakRef<MainWindow>,
}
#[derive(Debug)]
pub struct CoreRunningState {
    core: RunningCore,
    main_window_ref: SendWeakRef<MainWindow>,
    vcr_read_only: Cell<bool>,
    vcr_state: Arc<Mutex<Option<VcrState>>>,
}

struct CoreInputHandler {
    vcr_state: Arc<Mutex<Option<VcrState>>>,
    main_window_ref: SendWeakRef<MainWindow>,
}

struct CoreFrameHandler {
    vcr_state: Arc<Mutex<Option<VcrState>>>,
}

struct CoreSaveHandler {
    vcr_state: Arc<Mutex<Option<VcrState>>>,
}

impl Default for CoreState {
    fn default() -> Self {
        Self::Uninit
    }
}
impl From<CoreReadyState> for CoreState {
    fn from(value: CoreReadyState) -> Self {
        Self::Ready(value)
    }
}
impl From<CoreRunningState> for CoreState {
    fn from(value: CoreRunningState) -> Self {
        Self::Running(value)
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

    pub(super) fn borrow_running_mut(&mut self) -> Option<&mut CoreRunningState> {
        match self {
            CoreState::Running(running_state) => Some(running_state),
            _ => None,
        }
    }

    pub(super) fn borrow_running(&self) -> Option<&CoreRunningState> {
        match self {
            CoreState::Running(running_state) => Some(running_state),
            _ => None,
        }
    }

    pub(super) fn cfg_open(&self, name: &CStr) -> Result<ConfigSection<'_>, M64PError> {
        match self {
            CoreState::Uninit => panic!("Core is not initialized"),
            CoreState::Ready(ready_state) => ready_state.cfg_open(name),
            CoreState::Running(running_state) => running_state.cfg_open(name),
        }
    }
}

impl CoreReadyState {
    pub(super) fn new(main_window_ref: SendWeakRef<MainWindow>) -> Self {
        #[cfg(target_os = "windows")]
        const MUPEN_FILENAME: &str = "mupen64plus.dll";
        #[cfg(target_os = "macos")]
        const MUPEN_FILENAME: &str = "libmupen64plus.dylib";
        #[cfg(target_os = "linux")]
        const MUPEN_FILENAME: &str = "libmupen64plus.so";

        let mupen_dll_path = INSTALL_DIRS.core_dir.join(MUPEN_FILENAME);
        let data_dir = &INSTALL_DIRS.data_dir;

        let _ = fs::create_dir_all(&*CONFIG_DIR);

        log::info!("Loading M64+ from {}", mupen_dll_path.display());
        log::info!("Data path is {}", data_dir.display());

        // Load the core
        let mut core = m64prs_core::Core::init(mupen_dll_path, Some(&*CONFIG_DIR), Some(&data_dir))
            .expect("core startup should succeed");

        // Override the video functions to use the window compositor.
        let vidext_params = VideoExtensionParameters::new(main_window_ref.clone());
        core.override_vidext::<VideoExtensionState, _>(vidext_params)
            .expect("vidext override should succeed");

        {
            // Feed core events back to the GUI where needed.
            let main_window_ref = main_window_ref.clone();
            core.listen_state(move |param, value| match param {
                CoreParam::EmuState => {
                    let main_window_ref = main_window_ref.clone();
                    let _ = glib::spawn_future(async move {
                        main_window_ref.upgrade().inspect(|main_window| {
                            main_window.set_emu_state(
                                (value as <m64prs_sys::EmuState as TryFromPrimitive>::Primitive)
                                    .try_into()
                                    .unwrap(),
                            );
                        });
                    });
                }
                CoreParam::SavestateSlot => {
                    let main_window_ref = main_window_ref.clone();
                    let _ = glib::spawn_future(async move {
                        main_window_ref.upgrade().inspect(|main_window| {
                            main_window.set_save_slot(value.try_into().unwrap());
                        });
                    });
                }
                _ => (),
            });
        }

        // Apply the default config.
        settings_dialog::init_config(&mut core);

        {
            let main_window_ref = main_window_ref.clone();
            glib::spawn_future(async move {
                main_window_ref.upgrade().inspect(|main_window| {
                    main_window.set_emu_state(EmuState::Stopped);
                });
            });
        }

        Self {
            core,
            main_window_ref,
        }
    }

    pub(super) async fn start_rom<B>(
        mut self,
        rom_data: B,
        plugins: PluginSet,
    ) -> Result<CoreRunningState, (PluginLoadError, Self)>
    where
        B: Borrow<[u8]> + Send + 'static,
    {
        // Open ROM and attach plugins. This takes a bit.
        let mut core = {
            // transfer ownership of core to GIO task until it completes
            let mut core = self.core;
            let result = gio::spawn_blocking(move || {
                if let Err(err) = core.open_rom(rom_data.borrow()) {
                    return Err((PluginLoadError::M64P(err), core));
                }
                if let Err(err) = core.attach_plugins(plugins) {
                    core.close_rom().unwrap();
                    return Err((err, core));
                }
                Ok(core)
            })
            .await
            .unwrap();

            // If something bad happend while loading ROM or attaching plugins,
            // then return the error
            match result {
                Ok(core) => core,
                Err((err, core)) => {
                    self.core = core;
                    return Err((err, self));
                }
            }
        };

        // Load the ma
        let main_window_ref = self.main_window_ref;
        let vcr_state = Arc::new(Mutex::new(None));

        let input_handler = CoreInputHandler {
            vcr_state: Arc::clone(&vcr_state),
            main_window_ref: main_window_ref.clone(),
        };
        core.set_input_handler(input_handler)
            .expect("should be able to set input handler");

        let save_handler = CoreSaveHandler {
            vcr_state: Arc::clone(&vcr_state),
        };
        core.set_save_handler(save_handler)
            .expect("should be able to set save handler");

        let core = RunningCore::execute(core);

        Ok(CoreRunningState {
            core,
            main_window_ref,
            vcr_read_only: Cell::new(false),
            vcr_state,
        })
    }

    pub(super) fn cfg_open_mut(&mut self, name: &CStr) -> Result<ConfigSectionMut<'_>, M64PError> {
        self.core.cfg_open_mut(name)
    }

    pub(super) fn cfg_open(&self, name: &CStr) -> Result<ConfigSection<'_>, M64PError> {
        self.core.cfg_open(name)
    }
}

impl CoreRunningState {
    pub(super) async fn stop_rom(mut self) -> (CoreReadyState, Option<M64PError>) {
        let _ = self.unset_vcr_state().await;
        let (mut core, error) = gio::spawn_blocking(|| self.core.stop()).await.unwrap();

        let main_window_ref = self.main_window_ref;

        let _ = core.close_rom();
        let _ = core.detach_plugins();

        core.clear_input_handler()
            .expect("should be able to clear input handler");
        core.clear_save_handler()
            .expect("should be able to clear save handler");

        (
            CoreReadyState {
                core,
                main_window_ref,
            },
            error.err(),
        )
    }

    pub(super) fn toggle_pause(&self) -> Result<(), M64PError> {
        match self.core.emu_state() {
            EmuState::Running => self.core.request_pause(),
            EmuState::Paused => self.core.request_resume(),
            _ => unreachable!(),
        }
    }

    pub(super) fn frame_advance(&self) -> Result<(), M64PError> {
        self.core.request_advance_frame()
    }

    pub(super) fn reset(&self, hard: bool) -> Result<(), M64PError> {
        self.core.reset(hard)
    }

    pub(super) async fn save_slot(&self) -> Result<(), SavestateError> {
        self.core.save_slot().await
    }

    pub(super) async fn load_slot(&self) -> Result<(), SavestateError> {
        self.core.load_slot().await?;
        if let Some(vcr_state) = &mut *self.vcr_state.lock().await {
            vcr_state.set_read_only(self.vcr_read_only.get());
        }
        Ok(())
    }

    pub(super) fn set_save_slot(&self, slot: u8) -> Result<(), M64PError> {
        self.core.set_state_slot(slot)
    }

    pub(super) async fn save_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), SavestateError> {
        self.core
            .save_file(path.as_ref(), SavestateFormat::Mupen64Plus)
            .await
    }

    pub(super) async fn load_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), SavestateError> {
        self.core.load_file(path.as_ref()).await?;
        if let Some(vcr_state) = &mut *self.vcr_state.lock().await {
            vcr_state.set_read_only(self.vcr_read_only.get());
        }
        Ok(())
    }

    pub(super) fn forward_key_down(&self, key_code: u32, r#mod: gdk::ModifierType) {
        if let Some(window) = self.main_window_ref.upgrade() {
            let display = window.surface().unwrap().display();

            let sdl_key = keyboard::into_sdl_scancode(&display, key_code);
            let sdl_mod = keyboard::into_sdl_modifiers(r#mod);
            // eprintln!("0x{:02X} -> {:?}", key_code, sdl_key);
            let _ = self.core.forward_key_down(sdl_key, sdl_mod);
        }
    }

    pub(super) fn forward_key_up(&self, key_code: u32, r#mod: gdk::ModifierType) {
        if let Some(window) = self.main_window_ref.upgrade() {
            let display = window.surface().unwrap().display();

            let sdl_key = keyboard::into_sdl_scancode(&display, key_code);
            let sdl_mod = keyboard::into_sdl_modifiers(r#mod);
            // eprintln!("{:?} -> {:?}", key.name().unwrap().as_str(), sdl_key);
            let _ = self.core.forward_key_up(sdl_key, sdl_mod);
        }
    }

    fn notify_main_window<F: FnOnce(&MainWindow) + Send + 'static>(&self, f: F) {
        let main_window_ref = self.main_window_ref.clone();
        let _ = glib::spawn_future(async move {
            main_window_ref.upgrade().inspect(f);
        });
    }

    pub(super) async fn set_vcr_state(
        &self,
        mut vcr_state: VcrState,
        new: bool,
    ) -> Result<(), Box<dyn Error>> {
        vcr_state.set_read_only(self.vcr_read_only.get());
        vcr_state.reset(&self.core, new).await?;
        {
            let mut self_vcr_state = self.vcr_state.lock().await;
            *self_vcr_state = Some(vcr_state);
        }
        self.notify_main_window(|main_window| main_window.set_vcr_active(true));
        Ok(())
    }

    pub(super) async fn unset_vcr_state(&mut self) -> Option<VcrState> {
        let result = self.vcr_state.lock().await.take();
        self.notify_main_window(|main_window| main_window.set_vcr_active(false));
        result
    }

    pub(super) async fn export_vcr(&mut self) -> Option<(PathBuf, M64File)> {
        let vcr_state = self.vcr_state.lock().await;
        vcr_state.as_ref().map(|state| state.export())
    }

    pub(super) fn set_read_only(&self, value: bool) {
        self.vcr_read_only.set(value);
        self.notify_main_window(move |main_window| main_window.set_vcr_read_only(value));
    }

    pub(super) fn toggle_read_only(&self) {
        self.set_read_only(!self.vcr_read_only.get());
    }

    pub(super) fn rom_header(&self) -> RomHeader {
        self.core.rom_header().expect("couldn't get ROM header!")
    }

    pub(super) fn rom_settings(&self) -> RomSettings {
        self.core
            .rom_settings()
            .expect("couldn't get ROM settings!")
    }

    pub(super) fn plugin_info(&self, ptype: PluginType) -> PluginInfo {
        self.core.plugin_info(ptype).unwrap().unwrap()
    }

    pub(super) fn cfg_open(&self, name: &CStr) -> Result<ConfigSection<'_>, M64PError> {
        self.core.cfg_open(name)
    }
}

impl CoreInputHandler {
    fn notify_main_window<F: FnOnce(&MainWindow) + Send + 'static>(&self, f: F) {
        let main_window_ref = self.main_window_ref.clone();
        let _ = glib::spawn_future(async move {
            main_window_ref.upgrade().inspect(f);
        });
    }
}
impl InputHandler for CoreInputHandler {
    fn filter_inputs(
        &mut self,
        port: std::ffi::c_int,
        mut input: m64prs_sys::Buttons,
    ) -> m64prs_sys::Buttons {
        {
            let mut vcr_state = block_on(self.vcr_state.lock());
            let mut should_drop = false;
            if let Some(vcr_state) = vcr_state.as_mut() {
                (input, should_drop) = vcr_state.filter_inputs(port, input);
            }
            if should_drop {
                *vcr_state = None;
                self.notify_main_window(|main_window| main_window.set_vcr_active(false));
            }
        }
        input
    }

    fn poll_present(&mut self, port: std::ffi::c_int) -> bool {
        let mut vcr_state = block_on(self.vcr_state.lock());
        vcr_state
            .as_mut()
            .map_or(false, |state| state.poll_present(port))
    }
}

impl FrameHandler for CoreFrameHandler {
    fn new_frame(&mut self, _count: std::ffi::c_uint) {
        let mut vcr_state = block_on(self.vcr_state.lock());
        if let Some(vcr_state) = vcr_state.as_mut() {
            vcr_state.tick_vi();
        }
    }
}

impl SaveHandler for CoreSaveHandler {
    const SIGNATURE: u32 = u32::from_le_bytes([b'R', b'S', b'X', b'T']);

    fn save_xd(&mut self) -> Result<Box<[u8]>, Box<dyn Error>> {
        let mut vcr_state = block_on(self.vcr_state.lock());
        if let Some(vcr_state) = vcr_state.as_mut() {
            Ok(bincode::serialize(&vcr_state.freeze())?.into_boxed_slice())
        } else {
            Ok(Box::new([]))
        }
    }

    fn load_xd(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut vcr_state = block_on(self.vcr_state.lock());
        if let Some(vcr_state) = vcr_state.as_mut() {
            let freeze = bincode::deserialize(data)?;
            vcr_state.load_freeze(freeze)?;
        }

        Ok(())
    }
}
