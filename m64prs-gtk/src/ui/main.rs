use std::{cell::OnceCell, error::Error, path::PathBuf, sync::mpsc};

use gtk::{prelude::*, FileFilter};
use m64prs_sys::EmuState;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, WorkerController,
};

use crate::{
    controls,
    ui::dialogs::{
        alert::{AlertDialog, AlertDialogRequest, AlertDialogResponse, AlertDialogSettings},
        file::{FileDialog, FileDialogResponse, FileDialogSettings},
    },
};

use super::{
    actions::{self, *},
    core::{
        self,
        vidext::{VidextRequest, VidextResponse},
        MupenCore, MupenCoreRequest, MupenCoreResponse,
    },
    dialogs::file::FileDialogRequest,
};

#[derive(Debug)]
pub enum Message {
    NoOp,
    // MENU ITEMS
    // ==================

    // File
    MenuOpenRom,
    MenuOpenRom2(PathBuf),
    MenuCloseRom,
    // Emulator
    MenuTogglePause,
    MenuFrameAdvance,
    MenuResetRom,
    MenuSaveSlot,
    MenuLoadSlot,
    MenuSetSaveSlot(u8),

    // CORE CALLBACKS
    // ==================
    CoreReady {
        vidext_inbound: mpsc::Sender<(usize, VidextResponse)>,
    },
    CoreError(Box<dyn Error + Send + 'static>),
    CoreStateChanged(EmuState),
    CoreIoStateChanged(bool),
    CoreSavestateSlotChanged(u8),
    CoreVidextRequest(usize, VidextRequest),
}

#[derive(Debug, Clone, Copy)]
enum MainViewState {
    RomBrowser,
    GameView,
}

#[derive(Debug)]
pub struct Model {
    core: WorkerController<MupenCore>,
    core_ready: bool,
    core_state: EmuState,
    core_io_state: bool,
    core_savestate_slot: u8,
    vidext_inbound: OnceCell<mpsc::Sender<(usize, VidextResponse)>>,

    main_view: MainViewState,

    rom_file_dialog: Controller<FileDialog>,
    core_error_dialog: Controller<AlertDialog>,
}

impl Model {}

#[relm4::component(pub)]
impl Component for Model {
    type Input = Message;

    type Output = ();
    type Init = ();

    type CommandOutput = ();

    menu! {
        menu_root: {
            "File" {
                "Open ROM" => OpenRomAction,
                "Close ROM" => CloseRomAction,
            },
            "Emulator" {
                section! {
                    "Pause" => TogglePauseAction,
                    "Frame Advance" => FrameAdvanceAction,
                    "Reset ROM" => ResetRomAction,
                },
                section! {
                    "Save State" => SaveSlotAction,
                    "Load State" => LoadSlotAction,
                    "Current Slot" {
                        "1" => SetSaveSlotAction(1),
                        "2" => SetSaveSlotAction(2),
                        "3" => SetSaveSlotAction(3),
                        "4" => SetSaveSlotAction(4),
                        "5" => SetSaveSlotAction(5),
                        "6" => SetSaveSlotAction(6),
                        "7" => SetSaveSlotAction(7),
                        "8" => SetSaveSlotAction(8),
                        "9" => SetSaveSlotAction(9),
                    }
                }
            }
        }
    }

    view! {
        #[root]
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
        },
        #[name(app_actions)]
        actions::AppActions::new(&sender) {
            #[watch]
            set_core_state: model.core_state,
            #[watch]
            set_core_io_state: model.core_io_state,
            #[watch]
            set_core_savestate_slot: model.core_savestate_slot,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let core = MupenCore::builder()
            .detach_worker(())
            .forward(sender.input_sender(), |msg| match msg {
                MupenCoreResponse::CoreReady { vidext_inbound } => {
                    Message::CoreReady { vidext_inbound }
                }
                MupenCoreResponse::Error(error) => Message::CoreError(error),
                MupenCoreResponse::EmuStateChanged(emu_state) => {
                    Message::CoreStateChanged(emu_state)
                }
                MupenCoreResponse::VidextRequest(id, request) => {
                    Message::CoreVidextRequest(id, request)
                }
                MupenCoreResponse::StateRequestStarted => Message::CoreIoStateChanged(true),
                MupenCoreResponse::StateRequestComplete => Message::CoreIoStateChanged(false),
                MupenCoreResponse::SavestateSlotChanged(slot) => Message::CoreSavestateSlotChanged(slot),
            });

        let rom_file_dialog = FileDialog::builder()
            .launch(
                FileDialogSettings::new()
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
                FileDialogResponse::Accept(path) => Message::MenuOpenRom2(path),
                FileDialogResponse::Cancel => Message::NoOp,
            });
        let core_error_dialog = AlertDialog::builder()
            .launch(
                AlertDialogSettings::new()
                    .with_buttons(["OK"], 0, Some(0))
                    .with_transient_to(&root)
                    .with_modal(true),
            )
            .forward(sender.input_sender(), |msg| match msg {
                AlertDialogResponse::Choice(_) => Message::NoOp,
            });

        let model = Self {
            // core state
            core,
            core_ready: false,
            core_state: EmuState::Stopped,
            core_io_state: false,
            core_savestate_slot: 1,
            vidext_inbound: OnceCell::new(),
            // view state
            main_view: MainViewState::RomBrowser,
            // dialogs
            rom_file_dialog,
            core_error_dialog,
        };
        let widgets = view_output!();

        let app = relm4::main_application();
        log::info!(
            "Using GTK {}.{}.{}",
            gtk::major_version(),
            gtk::minor_version(),
            gtk::micro_version()
        );
        app.set_menubar(Some(&menu_root));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Message::NoOp => (),

            // MENU ACTIONS
            // ===============
            // File
            Message::MenuOpenRom => {
                self.rom_file_dialog.emit(FileDialogRequest::Open);
            }
            Message::MenuOpenRom2(path) => {
                self.core.emit(MupenCoreRequest::StartRom(path));
            }
            Message::MenuCloseRom => {
                self.core.emit(MupenCoreRequest::StopRom);
            }
            // Emulator
            Message::MenuTogglePause => {
                self.core.emit(MupenCoreRequest::TogglePause);
            }
            Message::MenuFrameAdvance => {
                self.core.emit(MupenCoreRequest::FrameAdvance);
            }
            Message::MenuResetRom => {
                self.core.emit(MupenCoreRequest::Reset);
            }
            // Shit
            Message::MenuSaveSlot => {
                self.core.emit(MupenCoreRequest::SaveSlot);
            }
            Message::MenuLoadSlot => {
                self.core.emit(MupenCoreRequest::LoadSlot);
            }
            Message::MenuSetSaveSlot(slot) => {
                self.core.emit(MupenCoreRequest::SetSaveSlot(slot));
            }
            // CORE FEEDBACK
            // ===============
            Message::CoreReady { vidext_inbound } => {
                self.core_ready = true;
                self.vidext_inbound.get_or_init(move || vidext_inbound);
            }
            Message::CoreError(error) => {
                const MESSAGE: &str = "Error occurred!";

                self.core_error_dialog.emit(AlertDialogRequest::Show {
                    message: MESSAGE.to_owned(),
                    detail: error.to_string(),
                })
            }
            Message::CoreStateChanged(emu_state) => {
                self.core_state = emu_state;
            }
            Message::CoreIoStateChanged(io_state) => {
                self.core_io_state = io_state;
            }
            Message::CoreSavestateSlotChanged(slot) => {
                self.core_savestate_slot = slot;
            }
            Message::CoreVidextRequest(id, request) => match request {
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
                VidextRequest::CreateView(attrs) => {
                    let vidext_inbound = self
                        .vidext_inbound
                        .get()
                        .expect("vidext request should be active");
                    let view = widgets.compositor.new_view(attrs);
                    let _ = vidext_inbound.send((id, VidextResponse::NewView(view)));
                }
                VidextRequest::DeleteView(view_key) => {
                    let vidext_inbound = self
                        .vidext_inbound
                        .get()
                        .expect("vidext request should be active");
                    widgets.compositor.del_view(view_key);
                    let _ = vidext_inbound.send((id, VidextResponse::Done));
                }
            },
        }

        self.update_view(widgets, sender);
    }
}
