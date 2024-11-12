use std::{error::Error, path::PathBuf};

use gtk::{prelude::*, FileFilter};
use m64prs_sys::EmuState;
use relm4::{
    actions::{RelmAction, RelmActionGroup}, Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent, WorkerController
};
use relm4_components::open_dialog::{OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings};

use crate::ui::{actions::*, core};

#[derive(Debug)]
pub enum Message {
    NoOp,
    // MENU ITEMS
    // ==================
    MenuRomOpen,
    MenuRomOpen2(PathBuf),

    MenuRomClose,
    // CORE CALLBACKS
    // ==================
    CoreReady,
    CoreError(Box<dyn Error + Send + 'static>),
    CoreStateChange(EmuState),
}

#[derive(Debug)]
pub struct Model {
    core: WorkerController<core::Model>,
    core_ready: bool,
    core_state: Option<EmuState>,

    rom_file_dialog: Controller<OpenDialog>
}

impl Model {
    fn register_menu_actions(sender: &ComponentSender<Self>) {
        let mut file_actions = RelmActionGroup::<AppActions>::new();
        file_actions.add_action(RelmAction::<OpenRomAction>::new_stateless({
            let sender = sender.clone();
            move |_| sender.input(Message::MenuRomOpen)
        }));
        file_actions.add_action(RelmAction::<CloseRomAction>::new_stateless({
            let sender = sender.clone();
            move |_| sender.input(Message::MenuRomClose)
        }));
        file_actions.register_for_main_application();
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Model {
    type Init = ();

    type Input = Message;
    type Output = ();

    menu! {
        menu_root: {
            "File" {
                "Open ROM" => OpenRomAction,
                "Close ROM" => CloseRomAction,
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
        let core = core::Model::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                core::Update::CoreReady => Message::CoreReady,
                core::Update::Error(error) => Message::CoreError(error),
                core::Update::EmuStateChange(emu_state) => {
                    Message::CoreStateChange(emu_state)
                }
            });

        let rom_file_dialog = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(OpenDialogSettings { 
                folder_mode: false, 
                cancel_label: "Cancel".into(), 
                accept_label: "OK".into(), 
                create_folders: true, 
                is_modal: true, 
                filters: vec![
                    {
                        let filter = FileFilter::new();
                        filter.set_name(Some("N64 ROMs (*.n64, *.v64, *.z64)"));

                        filter.add_pattern("*.n64");
                        filter.add_pattern("*.v64");
                        filter.add_pattern("*.z64");
                        
                        filter
                    }
                ],
            })
            .forward(sender.input_sender(), |msg| match msg {
                OpenDialogResponse::Accept(path) => Message::MenuRomOpen2(path),
                OpenDialogResponse::Cancel => Message::NoOp,
            });

        let model = Self {
            core,
            core_ready: false,
            core_state: None,
            rom_file_dialog
        };
        let widgets = view_output!();

        Self::register_menu_actions(&sender);
        let app = relm4::main_application();
        app.set_menubar(Some(&menu_root));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _: relm4::ComponentSender<Self>) {
        eprintln!("update thread: {:?}", std::thread::current().id());
        match message {
            Message::NoOp => (),

            Message::MenuRomOpen => {
                self.rom_file_dialog.emit(OpenDialogMsg::Open);
            }
            Message::MenuRomOpen2(path) => {
                self.core.emit(core::Request::StartRom(path));
            }

            Message::MenuRomClose => {
                self.core.emit(core::Request::StopRom);
            }

            Message::CoreReady => self.core_ready = true,
            Message::CoreError(_) => {},
            Message::CoreStateChange(emu_state) => self.core_state = Some(emu_state),
        }
    }
}
