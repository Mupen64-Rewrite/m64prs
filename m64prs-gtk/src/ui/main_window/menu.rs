use std::{error::Error, io};

use gtk::prelude::*;
use m64prs_core::{error::PluginLoadError, plugin::PluginSet, Plugin};
use m64prs_gtk_utils::actions::{BaseAction, StateAction, StateParamAction, TypedActionGroup};

use crate::ui::main_window::enums::MainEmuState;

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
    #[action(name = "vcr.discard_movie")]
    discard_movie: BaseAction,
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
    }

    fn bind_states(&self, main_window: &MainWindow) {
        let emu_state = main_window.property_expression_weak("emu-state");
        let saving_state = main_window.property_expression_weak("saving-state");
        let save_slot = main_window.property_expression_weak("save-slot");

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

        let save_start_valid = gtk::ClosureExpression::new::<bool>(
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

        let save_slot_gvar = save_slot.chain_closure::<glib::Variant>(glib::closure!(
            |_: Option<glib::Object>, save_slot: u8| -> glib::Variant { save_slot.into() }
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

        b!(save_slot."enabled" => save_start_valid);
        b!(load_slot."enabled" => save_start_valid);
        b!(set_save_slot."enabled" => emu_active);
        b!(set_save_slot."state" => save_slot_gvar);
        b!(save_file."enabled" => save_start_valid);
        b!(load_file."enabled" => save_start_valid);
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
            Ok::<_, PluginLoadError>(PluginSet {
                graphics: Plugin::load(plugin_path.join(plugin_name("mupen64plus-video-rice")))?,
                audio: Plugin::load(plugin_path.join(plugin_name("mupen64plus-audio-sdl")))?,
                input: Plugin::load(plugin_path.join(plugin_name("mupen64plus-input-sdl")))?,
                rsp: Plugin::load(plugin_path.join(plugin_name("mupen64plus-rsp-hle")))?,
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
    main_window.show_new_movie_dialog().await;
    Ok(())
}

async fn load_movie_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    main_window.show_load_movie_dialog().await;
    // let movie_file = match main_window.show_load_movie_dialog().await {
    //     Ok(file) => file,
    //     Err(err) => match err.kind::<gtk::DialogError>() {
    //         Some(gtk::DialogError::Dismissed) => return Ok(()),
    //         _ => return Err(err.into()),
    //     },
    // }
    // .read_future(glib::Priority::DEFAULT)
    // .await?
    // .into_async_buf_read(4096);

    // let movie = M64File::read_from_async(movie_file).await?;

    Ok(())
}