use std::error::Error;

use glib::Slice;
use gtk::prelude::*;
use m64prs_core::{error::PluginLoadError, plugin::PluginSet, Plugin};

use crate::{
    ui::main_window::enums::MainEmuState,
    utils::actions::{ActionGroupTypedExt, BaseAction, StateAction, StateParamAction},
};

use super::{CoreState, MainWindow};

pub fn load_menu() -> gio::MenuModel {
    const UI_XML: &str = gtk::gtk4_macros::include_blueprint!("src/ui/main_window/menu.blp");
    gtk::Builder::from_string(UI_XML)
        .object::<gio::MenuModel>("root")
        .expect("menu.blp should contain object `root`")
}

#[derive(Debug)]
pub(super) struct AppActions {
    open_rom: BaseAction,
    close_rom: BaseAction,

    toggle_pause: StateAction<bool>,
    frame_advance: BaseAction,
    reset_rom: BaseAction,

    save_slot: BaseAction,
    load_slot: BaseAction,
    set_save_slot: StateParamAction<u8, u8>,
    save_file: BaseAction,
    load_file: BaseAction,

    new_movie: BaseAction,
    load_movie: BaseAction,
    save_movie: BaseAction,
    discard_movie: BaseAction,
    toggle_read_only: StateAction<bool>,
}

impl Default for AppActions {
    fn default() -> Self {
        Self {
            open_rom: BaseAction::new("file.open_rom"),
            close_rom: BaseAction::new("file.close_rom"),
            toggle_pause: StateAction::new("emu.toggle_pause", false),
            frame_advance: BaseAction::new("emu.frame_advance"),
            reset_rom: BaseAction::new("emu.frame_advance"),
            save_slot: BaseAction::new("emu.frame_advance"),
            load_slot: BaseAction::new("emu.frame_advance"),
            set_save_slot: StateParamAction::new("emu.set_save_slot", 1),
            save_file: BaseAction::new("emu.frame_advance"),
            load_file: BaseAction::new("emu.frame_advance"),
            new_movie: BaseAction::new("emu.frame_advance"),
            load_movie: BaseAction::new("emu.frame_advance"),
            save_movie: BaseAction::new("emu.frame_advance"),
            discard_movie: BaseAction::new("emu.frame_advance"),
            toggle_read_only: StateAction::new("emu.toggle_read_only", false),
        }
    }
}

impl AppActions {
    pub(super) fn init(&self, main_window: &MainWindow) {
        self.connect_actions(main_window);
        self.bind_states(main_window);
    }

    fn connect_actions(&self, main_window: &MainWindow) {
        /// Binds a handler to one of the implementation functions.
        macro_rules! c {
            ($act:ident, async $handler:path, $msg:expr) => {
                self.$act.connect_activate({
                    let main_window = main_window.clone();
                    move |_| {
                        let main_window = main_window.clone();
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
            ($act:ident, $handler:path, $msg:expr) => {
                self.$act.connect_activate({
                    let main_window = main_window.clone();
                    move |_| {
                        let main_window = main_window.clone();
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
    }

    fn bind_states(&self, main_window: &MainWindow) {
        let (emu_stopped, emu_running) = {
            let this = gtk::ObjectExpression::new(main_window);
            let emu_state =
                gtk::PropertyExpression::new(MainWindow::static_type(), Some(this), "emu-state");

            let emu_stopped = gtk::ClosureExpression::new::<bool>(
                [emu_state.clone()],
                glib::closure!(|_: Option<glib::Object>, emu_state: MainEmuState| -> bool {
                    matches!(emu_state, MainEmuState::Stopped)
                }),
            );

            let emu_running = gtk::ClosureExpression::new::<bool>(
                [emu_state.clone()],
                glib::closure!(|_: Option<glib::Object>, emu_state: MainEmuState| -> bool {
                    matches!(emu_state, MainEmuState::Running | MainEmuState::Paused)
                }),
            );

            (emu_stopped, emu_running)
        };

        /// Bind an action's property to an expression.
        macro_rules! b {
            ($name:ident.$prop:literal = $expr:ident) => {
                $expr.bind(self.$name.inner(), $prop, None::<&glib::Object>);
            };
        }

        b!(open_rom."enabled" = emu_stopped);
        b!(close_rom."enabled" = emu_running);
    }

    pub(super) fn register_to(&self, map: &impl IsA<gio::ActionMap>) {
        macro_rules! register_all_actions {
            ($($names:ident),* $(,)?) => {
                {
                    $(map.register_action(&self.$names);)*
                }
            };
        }
        register_all_actions!(
            open_rom,
            close_rom,
            toggle_pause,
            frame_advance,
            reset_rom,
            save_slot,
            load_slot,
            set_save_slot,
            save_file,
            load_file,
            new_movie,
            load_movie,
            save_movie,
            discard_movie,
            toggle_read_only
        );
    }
}

// HELPERS
// ================

// IMPLEMENTATIONS
// =====================

async fn open_rom_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    let rom_file = main_window.show_open_rom_dialog().await?;

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
    main_window.exec_with_core(move |core| {
        let ready = match core.take() {
            CoreState::Ready(ready_state) => ready_state,
            _ => panic!("Expected Ready state"),
        };

        match ready.start_rom(&rom_data, plugins) {
            Ok(running) => {
                *core = CoreState::Running(running);
                Ok(())
            }
            Err((error, ready)) => {
                *core = CoreState::Ready(ready);
                Err(error)
            }
        }
    })?;

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
        .toggle_pause()
        .expect("Command should succeed");
    Ok(())
}

fn frame_advance_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .frame_advance()
        .expect("Command should succeed");
    Ok(())
}

fn reset_rom_impl(main_window: &MainWindow) -> Result<(), Box<dyn Error>> {
    
    main_window
        .borrow_core()
        .borrow_running()
        .expect("Core should be running")
        .reset(false)
        .expect("Command should succeed");
    Ok(())
}
