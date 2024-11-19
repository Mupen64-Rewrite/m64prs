use std::{
    cell::{OnceCell, RefCell},
    error::Error,
    path::PathBuf,
    sync::mpsc,
};

use gtk::{prelude::*, FileFilter};
use m64prs_sys::EmuState;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    WorkerController,
};

use crate::{
    controls::{self},
    ui::{actions::*, core},
};

use super::{
    alert_dialog,
    core::vidext::{VidextRequest, VidextResponse},
    file_dialog,
};

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
    CoreReady {
        vidext_inbound: mpsc::Sender<(usize, VidextResponse)>,
    },
    CoreError(Box<dyn Error + Send + 'static>),
    CoreStateChange(EmuState),
    CoreVidextRequest(usize, VidextRequest),
}

#[derive(Debug, Clone, Copy)]
enum MainViewState {
    RomBrowser,
    GameView,
}

#[derive(Debug)]
pub struct Model {
    core: WorkerController<core::Model>,
    core_ready: bool,
    core_state: Option<EmuState>,
    vidext_inbound: OnceCell<mpsc::Sender<(usize, VidextResponse)>>,

    main_view: MainViewState,
    vidext_request: RefCell<Option<(usize, VidextRequest)>>,

    rom_file_dialog: Controller<file_dialog::Model>,
    core_error_dialog: Controller<alert_dialog::Model>,
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
    type Input = Message;

    type Output = ();
    type Init = ();

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
        #[name(root)]
        gtk::ApplicationWindow::new(&relm4::main_application()) {
            set_title: Some("m64prs"),
            set_default_width: -1,
            set_default_height: -1,
            set_show_menubar: true,
            set_size_request: (200, 200),

            match model.main_view {
                MainViewState::RomBrowser => gtk::Button::with_label("test") {
                    set_hexpand: true,
                    set_vexpand: true,
                },
                MainViewState::GameView =>
                #[name(compositor)]
                controls::compositor_view::CompositorView {
                    set_hexpand: true,
                    set_vexpand: true,
                }
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
                core::Response::CoreReady { vidext_inbound } => {
                    Message::CoreReady { vidext_inbound }
                }
                core::Response::Error(error) => Message::CoreError(error),
                core::Response::EmuStateChange(emu_state) => Message::CoreStateChange(emu_state),
                core::Response::VidextRequest(id, request) => {
                    Message::CoreVidextRequest(id, request)
                }
            });

        let rom_file_dialog = file_dialog::Model::builder()
            .launch(
                file_dialog::Settings::new()
                    .with_transient_to(&root)
                    .with_title("Open ROM...")
                    .with_filters(
                        vec![{
                            let filter = FileFilter::new();
                            filter.set_name(Some("N64 ROMs (*.n64, *.v64, *.z64)"));

                            filter.add_pattern("*.n64");
                            filter.add_pattern("*.v64");
                            filter.add_pattern("*.z64");

                            filter
                        }],
                        Some(0),
                    ),
            )
            .forward(sender.input_sender(), |msg| match msg {
                file_dialog::Response::Accept(path) => Message::MenuRomOpen2(path),
                file_dialog::Response::Cancel => Message::NoOp,
            });
        let core_error_dialog = alert_dialog::Model::builder()
            .launch(alert_dialog::Settings::new().with_buttons(["OK"], 0, Some(0)))
            .forward(sender.input_sender(), |msg| match msg {
                alert_dialog::Response::Choice(_) => Message::NoOp,
            });

        let model = Self {
            // core state
            core,
            core_ready: false,
            core_state: None,
            vidext_inbound: OnceCell::new(),
            // view state
            main_view: MainViewState::RomBrowser,
            vidext_request: RefCell::new(None),
            // dialogs
            rom_file_dialog,
            core_error_dialog,
        };
        let widgets = view_output!();

        Self::register_menu_actions(&sender);
        let app = relm4::main_application();
        log::info!("Using GTK {}.{}.{}", gtk::major_version(), gtk::minor_version(), gtk::micro_version());
        app.set_menubar(Some(&menu_root));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match message {
            Message::NoOp => (),

            Message::MenuRomOpen => {
                self.rom_file_dialog.emit(file_dialog::Request::Open);
            }
            Message::MenuRomOpen2(path) => {
                self.core.emit(core::Request::StartRom(path));
            }

            Message::MenuRomClose => {
                self.core.emit(core::Request::StopRom);
            }

            Message::CoreReady { vidext_inbound } => {
                self.core_ready = true;
                self.vidext_inbound.get_or_init(move || vidext_inbound);
            }
            Message::CoreError(error) => {
                const MESSAGE: &str = "Error occurred!";

                self.core_error_dialog.emit(alert_dialog::Request::Show {
                    message: MESSAGE.to_owned(),
                    detail: error.to_string(),
                })
            }
            Message::CoreStateChange(emu_state) => self.core_state = Some(emu_state),
            Message::CoreVidextRequest(id, request) => {
                match request {
                    VidextRequest::EnterGameView => {
                        let vidext_inbound = self
                            .vidext_inbound
                            .get()
                            .expect("vidext request should be active");

                        self.main_view = MainViewState::GameView;
                        let _ = vidext_inbound.send((id, VidextResponse::Done));
                    }
                    VidextRequest::ExitGameView => {
                        let vidext_inbound = self
                            .vidext_inbound
                            .get()
                            .expect("vidext request should be active");

                        self.main_view = MainViewState::RomBrowser;
                        let _ = vidext_inbound.send((id, VidextResponse::Done));
                    }
                    _ => {
                        // Some requests can only be handled during the view update
                        self.vidext_request.replace(Some((id, request)));
                    }
                }
            }
        }
    }

    fn post_view() {
        let vidext_inbound = self
            .vidext_inbound
            .get()
            .expect("vidext request should be active");

        // Handle view-update requests (subsurfaces)
        let mut vidext_request = self.vidext_request.borrow_mut();
        if let Some((id, request)) = vidext_request.take() {
            match request {
                VidextRequest::CreateView(attrs) => {
                    let view = compositor.new_view(attrs);
                    let _ = vidext_inbound.send((id, VidextResponse::NewView(view)));
                }
                VidextRequest::DeleteView(view_key) => {
                    compositor.del_view(view_key);
                    let _ = vidext_inbound.send((id, VidextResponse::Done));
                }
                _ => (),
            }
        }
    }
}
