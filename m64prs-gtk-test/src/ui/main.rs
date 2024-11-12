use std::{error::Error, path::PathBuf};

use gtk::prelude::*;
use m64prs_sys::EmuState;
use relm4::{
    actions::RelmAction, Component, ComponentController, ComponentParts, Controller, SimpleComponent
};

use crate::ui::{actions::*, core_worker};

#[derive(Debug)]
pub enum Message {
    // MENU ITEMS
    // ==================
    MenuRomOpen,
    MenuRomOpen2(PathBuf),
    // CORE CALLBACKS
    // ==================
    CoreReady,
    CoreError(Box<dyn Error + Send + 'static>),
    CoreStateChange(EmuState),
}

#[derive(Debug)]
pub struct Model {
    core: Controller<core_worker::Model>,
    core_ready: bool,
    core_state: Option<EmuState>,
}

impl Model {
}

#[relm4::component(pub)]
impl SimpleComponent for Model {
    type Init = ();

    type Input = Message;
    type Output = ();

    menu! {
        menu_root: {
            "File" {
                "Open ROM" => RomOpenAction,
                "Close ROM" => RomCloseAction,
            },
            "Emulator" {
                "Pause" => TogglePauseAction,
            }
        }
    }

    view! {
        gtk::ApplicationWindow::new(&relm4::main_application()) {
            set_title: Some("m64prs"),
            set_default_width: -1,
            set_default_height: -1,
            set_show_menubar: true,

            gtk::Stack {
                set_size_request: (640, 480)
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let core = core_worker::Model::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                core_worker::Update::CoreReady => Message::CoreReady,
                core_worker::Update::Error(error) => Message::CoreError(error),
                core_worker::Update::EmuStateChange(emu_state) => {
                    Message::CoreStateChange(emu_state)
                }
            });

        let model = Self {
            core: core,
            core_ready: false,
            core_state: None
        };
        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_menubar(Some(&menu_root));

        let open_rom_action = RelmAction::<RomOpenAction>::new_stateless({
            let sender = sender.clone();
            move |_|  {
                sender.input(Message::MenuRomOpen);
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _: relm4::ComponentSender<Self>) {
        match message {
            Message::MenuRomOpen => {

            }
            Message::MenuRomOpen2(path) => {
                self.core.sender().emit(core_worker::Request::StartRom(path));
            }

            Message::CoreReady => self.core_ready = true,
            Message::CoreError(_) => {},
            Message::CoreStateChange(emu_state) => self.core_state = Some(emu_state),
        }
    }

    fn post_view() {
        
    }
}
