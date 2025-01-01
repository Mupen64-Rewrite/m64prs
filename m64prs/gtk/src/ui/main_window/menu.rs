use std::{error::Error, io, path::Path};

use gtk::prelude::*;
use m64prs_core::{
    error::PluginLoadError,
    plugin::{PluginSet, PluginType},
    Plugin,
};
use m64prs_gtk_utils::{
    actions::{BaseAction, StateAction, StateParamAction, TypedActionGroup},
    error::GlibErrorExt,
};
use m64prs_vcr::{movie::M64File, VcrState};

use crate::{ui::main_window::enums::MainEmuState, utils::dirs::INSTALL_DIRS};

use super::{CoreState, MainWindow};

pub fn load_menu() -> gio::MenuModel {
    const UI_XML: &str = include_str!("menu.ui");
    gtk::Builder::from_string(UI_XML)
        .object::<gio::MenuModel>("root")
        .expect("menu.blp should contain object `root`")
}

#[derive(Debug, TypedActionGroup)]
pub(super) struct AppActions {
    #[action(name = "file.open_rom")]
    open_rom: BaseAction,
    #[action(name = "file.close_rom")]
    close_rom: BaseAction,

    #[action(name = "emu.toggle_pause", default = false)]
    toggle_pause: StateAction<bool>,
    #[action(name = "emu.frame_advance")]
    frame_advance: BaseAction,
    #[action(name = "emu.reset_rom")]
    reset_rom: BaseAction,

    #[action(name = "emu.save_slot")]
    save_slot: BaseAction,
    #[action(name = "emu.load_slot")]
    load_slot: BaseAction,
    #[action(name = "emu.set_save_slot", default = 1u8)]
    set_save_slot: StateParamAction<u8, u8>,
    #[action(name = "emu.save_file")]
    save_file: BaseAction,
    #[action(name = "emu.load_file")]
    load_file: BaseAction,

    #[action(name = "vcr.new_movie")]
    new_movie: BaseAction,
    #[action(name = "vcr.load_movie")]
    load_movie: BaseAction,
    #[action(name = "vcr.save_movie")]
    save_movie: BaseAction,
    #[action(name = "vcr.close_movie")]
    close_movie: BaseAction,
    #[action(name = "vcr.toggle_read_only", default = false)]
    toggle_read_only: StateAction<bool>,
}

impl Default for AppActions {
    fn default() -> Self {
        Self::new_default()
    }
}

impl AppActions {
    pub(super) fn init(&self, main_window: &MainWindow) {
        self.connect_actions(main_window);
        self.bind_states(main_window);
    }

    fn connect_actions(&self, main_window: &MainWindow) {
        macro_rules! c {
            ($act:ident, async @$handler:path, $msg:expr) => {
                self.$act.connect_activate({
                    let main_window = main_window.downgrade();
                    move |_, param| {
                        let main_window = main_window
                            .upgrade()
                            .expect("Failed to get main window ref");
                        ::glib::spawn_future_local(async move {
                            if let Err(err) = $handler(&main_window, param).await {
                                main_window
                                    .show_error_dialog($msg, &*err)
                                    .await;
                            }
                        });
                    }
                })
            };
            ($act:ident, async @$handler:path) => {
                c!($act, async $handler, "Operation failed!");
            };
            ($act:ident, async $handler:path, $msg:expr) => {
                self.$act.connect_activate({
                    let main_window = main_window.downgrade();
                    move |_| {
                        let main_window = main_window
                            .upgrade()
                            .expect("Failed to get main window ref");
                        ::glib::spawn_future_local(async move {
                            if let Err(err) = $handler(&main_window).await {
                                main_window
                                    .show_error_dialog($msg, &*err)
                                    .await;
                            }
                        });
                    }
                })
            };
            ($act:ident, async $handler:path) => {
                c!($act, async $handler, "Operation failed!");
            };
            ($act:ident, @$handler:path, $msg:expr) => {
                self.$act.connect_activate({
                    let main_window = main_window.downgrade();
                    move |_, param| {
                        let main_window = main_window
                            .upgrade()
                            .expect("Failed to get main window ref");
                        ::glib::spawn_future_local(async move {
                            if let Err(err) = $handler(&main_window, param) {
                                main_window
                                    .show_error_dialog($msg, &*err)
                                    .await;
                            }
                        });
                    }
                })
            };
            ($act:ident, @$handler:path) => {
                c!($act, @$handler, "Operation failed!")
            };
            ($act:ident, $handler:path, $msg:expr) => {
                self.$act.connect_activate({
                    let main_window = main_window.downgrade();
                    move |_| {
                        let main_window = main_window
                            .upgrade()
                            .expect("Failed to get main window ref");
                        ::glib::spawn_future_local(async move {
                            if let Err(err) = $handler(&main_window) {
                                main_window
                                    .show_error_dialog($msg, &*err)
                                    .await;
                            }
                        });
                    }
                })
            };
            ($act:ident, $handler:path) => {
                c!($act, $handler, "Operation failed!");
            };
        }

        c!(open_rom, async open_rom_impl);
        c!(close_rom, async close_rom_impl);

        c!(toggle_pause, toggle_pause_impl);
        c!(frame_advance, frame_advance_impl);
        c!(reset_rom, reset_rom_impl);

        c!(save_slot, async save_slot_impl);
        c!(load_slot, async load_slot_impl);
        c!(set_save_slot, @set_save_slot_impl);
        c!(save_file, async save_file_impl);
        c!(load_file, async load_file_impl);

        c!(new_movie, async new_movie_impl);
        c!(load_movie, async load_movie_impl);
        c!(save_movie, async save_movie_impl);
        c!(close_movie, async close_movie_impl);
        c!(toggle_read_only, toggle_read_only_impl);
    }

    fn bind_states(&self, main_window: &MainWindow) {
        let emu_state = main_window.property_expression_weak("emu-state");
        let saving_state = main_window.property_expression_weak("saving-state");
        let save_slot = main_window.property_expression_weak("save-slot");
        let vcr_active = main_window.property_expression_weak("vcr-active");
        let vcr_read_only = main_window.property_expression_weak("vcr-read-only");

        let emu_stopped =
            emu_state.chain_closure::<bool>(glib::closure!(|_: Option<glib::Object>,
                                                            emu_state: MainEmuState|
             -> bool {
                matches!(emu_state, MainEmuState::Stopped)
            }));

        let emu_active =
            emu_state.chain_closure::<bool>(glib::closure!(|_: Option<glib::Object>,
                                                            emu_state: MainEmuState|
             -> bool {
                matches!(emu_state, MainEmuState::Running | MainEmuState::Paused)
            }));

        let emu_paused_gvar = emu_state.chain_closure::<glib::Variant>(glib::closure!(
            |_: Option<glib::Object>, emu_state: MainEmuState| -> glib::Variant {
                matches!(emu_state, MainEmuState::Paused).into()
            }
        ));

        let can_save = gtk::ClosureExpression::new::<bool>(
            [&*emu_state, &*saving_state],
            glib::closure!(|_: Option<glib::Object>,
                            emu_state: MainEmuState,
                            saving_state: bool|
             -> bool {
                matches!(
                    (emu_state, saving_state),
                    (MainEmuState::Running | MainEmuState::Paused, false)
                )
            }),
        );
        let has_vcr = gtk::ClosureExpression::new::<bool>(
            [&*emu_state, &*vcr_active],
            glib::closure!(|_: Option<glib::Object>,
                            emu_state: MainEmuState,
                            vcr_active: bool|
             -> bool {
                matches!(
                    (emu_state, vcr_active),
                    (MainEmuState::Running | MainEmuState::Paused, true)
                )
            }),
        );

        let save_slot_gvar = save_slot.chain_closure::<glib::Variant>(glib::closure!(
            |_: Option<glib::Object>, save_slot: u8| -> glib::Variant { save_slot.into() }
        ));
        let vcr_read_only_gvar = vcr_read_only.chain_closure::<glib::Variant>(glib::closure!(
            |_: Option<glib::Object>, read_only: bool| -> glib::Variant { read_only.into() }
        ));

        /// Bind an action's property to an expression.
        macro_rules! b {
            ($name:ident.$prop:literal => $expr:ident) => {{
                $expr.bind(self.$name.inner(), $prop, None::<&glib::Object>);
            }};
        }

        b!(open_rom."enabled" => emu_stopped);
        b!(close_rom."enabled" => emu_active);

        b!(toggle_pause."enabled" => emu_active);
        b!(toggle_pause."state" => emu_paused_gvar);
        b!(frame_advance."enabled" => emu_active);
        b!(reset_rom."enabled" => emu_active);

        b!(save_slot."enabled" => can_save);
        b!(load_slot."enabled" => can_save);
        b!(set_save_slot."enabled" => emu_active);
        b!(set_save_slot."state" => save_slot_gvar);
        b!(save_file."enabled" => can_save);
        b!(load_file."enabled" => can_save);

        b!(new_movie."enabled" => emu_active);
        b!(load_movie."enabled" => emu_active);
        b!(save_movie."enabled" => has_vcr);
        b!(close_movie."enabled" => has_vcr);
        b!(toggle_read_only."enabled" => emu_active);
        b!(toggle_read_only."state" => vcr_read_only_gvar);
    }
}

// HELPERS
// =====================

/// Helper function to lock savestate actions while
/// one is already happening.
struct SaveOpGuard<'a> {
    main_window: &'a MainWindow,
}
impl<'a> SaveOpGuard<'a> {
    fn new(main_window: &'a MainWindow) -> Self {
        main_window.set_saving_state(true);
        Self { main_window }
    }
}
impl<'a> Drop for SaveOpGuard<'a> {
    fn drop(&mut self) {
        self.main_window.set_saving_state(false);
    }
}

// IMPLEMENTATIONS
// =====================

async fn open_rom_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let rom_file = match main_window.show_open_rom_dialog().await {
        Ok(file) => file,
        Err(err) => match err.kind::<gtk::DialogError>() {
            Some(gtk::DialogError::Dismissed) => return Ok(()),
            _ => return Err(err.into()),
        },
    };

    // setup futures for loading ROM data and plugins
    let rom_data_fut =
        async move { Ok::<_, glib::Error>(rom_file.load_contents_future().await?.0) };
    let plugin_fut = async {
        gio::spawn_blocking(|| {
            let plugin_dir: &Path = &INSTALL_DIRS.plugin_dir;
            println!("plugins: {}", plugin_dir.display());

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
            Ok::<_, PluginLoadError>(PluginSet {
                graphics: Plugin::load(plugin_dir.join(plugin_name("mupen64plus-video-rice")))?,
                audio: Plugin::load(plugin_dir.join(plugin_name("mupen64plus-audio-sdl")))?,
                input: Plugin::load(plugin_dir.join(plugin_name("mupen64plus-input-tasinput")))?,
                rsp: Plugin::load(plugin_dir.join(plugin_name("mupen64plus-rsp-hle")))?,
            })
        })
        .await
        .unwrap()
    };

    // await both
    let (rom_data, plugins) = futures::join!(rom_data_fut, plugin_fut);
    let (rom_data, plugins) = (rom_data?, plugins?);

    // start the core
    {
        let mut core = main_window.borrow_core();

        let ready = match core.take() {
            CoreState::Ready(ready_state) => ready_state,
            _ => panic!("Expected Ready state"),
        };

        match ready.start_rom(rom_data, plugins).await {
            Ok(running) => {
                *core = CoreState::Running(running);
                Ok(())
            }
            Err((error, ready)) => {
                *core = CoreState::Ready(ready);
                Err(error)
            }
        }?;
    }

    Ok(())
}

async fn close_rom_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let mut core = main_window.borrow_core();

    let running = match core.take() {
        CoreState::Running(running_state) => running_state,
        _ => panic!("Expected Running state"),
    };

    let (ready, error) = running.stop_rom().await;

    *core = CoreState::Ready(ready);

    if let Some(error) = error {
        return Err(error.into());
    }
    Ok(())
}

fn toggle_pause_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .toggle_pause()?;
    Ok(())
}

fn frame_advance_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .frame_advance()?;
    Ok(())
}

fn reset_rom_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .reset(false)?;
    Ok(())
}

async fn save_slot_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let _guard = SaveOpGuard::new(main_window);
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .save_slot()
        .await?;
    Ok(())
}

async fn load_slot_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let _guard = SaveOpGuard::new(main_window);
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .load_slot()
        .await?;
    Ok(())
}

// TODO: switch out String param for u8 once blueprint supports it.
fn set_save_slot_impl(main_window: &MainWindow, slot: u8) -> Result<(), Box<dyn Error>> {
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .set_save_slot(slot)?;
    Ok(())
}

async fn save_file_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let save_file = match main_window.show_save_state_dialog().await {
        Ok(file) => file,
        Err(err) => match err.kind::<gtk::DialogError>() {
            Some(gtk::DialogError::Dismissed) => return Ok(()),
            _ => return Err(err.into()),
        },
    };

    let path = save_file
        .path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Couldn't get path to savestate"))?;

    {
        let _guard = SaveOpGuard::new(main_window);
        main_window
            .borrow_core()
            .borrow_running()
            .expect("Core should be running")
            .save_file(path)
            .await?;
    }

    Ok(())
}

async fn load_file_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let save_file = match main_window.show_load_state_dialog().await {
        Ok(file) => file,
        Err(err) => match err.kind::<gtk::DialogError>() {
            Some(gtk::DialogError::Dismissed) => return Ok(()),
            _ => return Err(err.into()),
        },
    };

    let path = save_file
        .path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Couldn't get path to savestate"))?;

    {
        let _guard = SaveOpGuard::new(main_window);
        main_window
            .borrow_core()
            .borrow_running()
            .expect("Core should be running")
            .load_file(path)
            .await?;
    }

    Ok(())
}

async fn new_movie_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let (path, mut header) = match main_window.show_new_movie_dialog().await {
        Some(file) => file,
        None => return Ok(()),
    };

    {
        let mut core_ref = main_window.borrow_core();
        let core = core_ref.borrow_running().expect("Core should be running");

        let rom_header = core.rom_header();
        header.rom_cc = rom_header.Country_code as u16;
        header.rom_crc = rom_header.CRC1;

        let _ = header.graphics_plugin.write_clipped(
            &core
                .plugin_info(PluginType::Graphics)
                .plugin_name
                .to_string_lossy(),
        );
        let _ = header.audio_plugin.write_clipped(
            &core
                .plugin_info(PluginType::Audio)
                .plugin_name
                .to_string_lossy(),
        );
        let _ = header.input_plugin.write_clipped(
            &core
                .plugin_info(PluginType::Input)
                .plugin_name
                .to_string_lossy(),
        );
        let _ = header.rsp_plugin.write_clipped(
            &core
                .plugin_info(PluginType::Rsp)
                .plugin_name
                .to_string_lossy(),
        );

        let vcr_state = VcrState::new(path, header, false);
        core.set_read_only(false);
        println!("setting VCR state");
        core.set_vcr_state(vcr_state, true).await?;
    };

    Ok(())
}

async fn load_movie_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let movie_file = match main_window.show_load_movie_dialog().await {
        Some(file) => file,
        None => return Ok(()),
    };
    let reader = movie_file
        .read_future(glib::Priority::DEFAULT)
        .await?
        .into_async_buf_read(4096);

    let movie = M64File::read_from_async(reader).await?;

    {
        let mut core_ref = main_window.borrow_core();
        let core = core_ref.borrow_running().expect("Core should be running");
        let vcr_state = VcrState::with_m64(movie_file.path().unwrap(), movie, true);
        core.set_read_only(true);
        core.set_vcr_state(vcr_state, false).await?;
    }

    Ok(())
}

async fn save_movie_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let exported = main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .export_vcr()
        .await;

    if let Some((path, data)) = exported {
        gio::spawn_blocking(move || -> Result<(), io::Error> {
            let out_file = gio::File::for_path(&path);
            let out_iostream = out_file
                .open_readwrite(None::<&gio::Cancellable>)
                .map_err(|err| err.try_into_io_error().unwrap())?;
            let mut out_ostream = out_iostream.output_stream().into_write();

            data.write_into(&mut out_ostream)?;

            Ok(())
        })
        .await
        .unwrap()?;
    }

    Ok(())
}

async fn close_movie_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let vcr_state = main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .unset_vcr_state()
        .await;
    let vcr_read_only = main_window.vcr_read_only();

    // Only save if we're not in read-only mode.
    if let (false, Some(vcr_state)) = (vcr_read_only, vcr_state) {
        // GIO async doesn't work for us. Move the data to a blocking task,
        // then await that instead.
        let (path, data) = vcr_state.export();
        gio::spawn_blocking(move || -> Result<(), io::Error> {
            let out_file = gio::File::for_path(&path);
            let mut out_stream = out_file
                .open_readwrite(None::<&gio::Cancellable>)
                .map_err(|err| err.try_into_io_error().unwrap())?
                .output_stream()
                .into_write();

            data.write_into(&mut out_stream)?;

            Ok(())
        })
        .await
        .unwrap()?;
    }

    Ok(())
}

fn toggle_read_only_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let _ = main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .toggle_read_only();
    Ok(())
}
