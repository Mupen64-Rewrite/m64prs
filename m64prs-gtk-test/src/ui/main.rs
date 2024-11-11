use gtk::prelude::*;
use relm4::{actions::RelmActionGroup, Component, ComponentParts, Controller, RelmWidgetExt, SimpleComponent};

use crate::ui::{actions::*, core_worker};

#[derive(Debug)]
pub enum Message {
    CoreReady,
}

#[derive(Debug)]
pub struct Model {
    core: Controller<core_worker::Model>,
    core_ready: bool,
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
            });


        let model = Self {
            core: core,
            core_ready: false,
        };
        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_menubar(Some(&menu_root));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _: relm4::ComponentSender<Self>) {
        match message {
            Message::CoreReady => self.core_ready = true,
        }
    }
}
