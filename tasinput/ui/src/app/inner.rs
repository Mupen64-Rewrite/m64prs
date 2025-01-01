use std::{cell::RefCell, mem};

use glib::SendWeakRef;
use gtk::{prelude::*, subclass::prelude::*};
use tasinput_protocol::{Endpoint, HostMessage, UiMessage};
use tokio::{
    sync::mpsc::{self},
    time::Instant,
};

use crate::{
    app::{FLAG_SOCKET, SOCKET_ID_KEY},
    main_window::MainWindow,
};

use super::endpoint::{ping_timeout, UiRequestHandler};

#[derive(Default)]
pub struct TasDiApp {
    endpoint: RefCell<Option<Endpoint<UiMessage, HostMessage>>>,
    pub(super) windows: RefCell<[Option<MainWindow>; 4]>,
}

impl TasDiApp {
    fn load_css(&self) {
        const CSS_CONTENT: &'static str =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/main.css"));

        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_string(CSS_CONTENT);

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn connect_endpoint(&self) {
        let mut endpoint = self.endpoint.borrow_mut();

        // Increment the reference count. We will explicitly quit the app
        // when the socket server tells us to do so.
        mem::forget(self.obj().hold());

        // extract the socket key from CLI
        let socket_id = unsafe { self.obj().steal_data::<String>(SOCKET_ID_KEY).unwrap() };

        let app_ref: SendWeakRef<_> = self.obj().downgrade().into();
        *endpoint = Some({
            // This channel keeps a record of ping expiry times. Whenever a ping is sent,
            // its expiry time is put on the channel. If the receiving task depletes this queue,
            // then the host has probably crashed and we should shut down.
            let (timeout_send, timeout_recv) = mpsc::channel::<Instant>(16);

            // Main endpoint request handler.
            let mut endpoint = Endpoint::<UiMessage, HostMessage>::connect(
                &socket_id,
                UiRequestHandler {
                    app_ref: app_ref.clone(),
                    timeout_send,
                },
            )
            .unwrap();

            // Ping timeout handler.
            endpoint.spawn({
                let app_ref = app_ref.clone();
                |handle| ping_timeout(app_ref, timeout_recv, handle.cancel_token())
            });

            endpoint
        });
    }

    fn show_ui(&self) {
        let window = MainWindow::new(&*self.obj());
        window.present();
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TasDiApp {
    const NAME: &'static str = "TasDiApp";
    type Type = super::TasDiApp;
    type ParentType = gtk::Application;
}

impl ObjectImpl for TasDiApp {}
impl ApplicationImpl for TasDiApp {
    fn command_line(&self, command_line: &gio::ApplicationCommandLine) -> glib::ExitCode {
        self.parent_command_line(command_line);
        let options = command_line.options_dict();
        let socket = options.lookup::<String>(FLAG_SOCKET).unwrap();

        if let Some(socket) = socket {
            unsafe {
                self.obj().set_data::<String>(SOCKET_ID_KEY, socket);
            }
        }
        self.obj().activate();

        glib::ExitCode::SUCCESS
    }

    fn startup(&self) {
        self.parent_startup();
        self.load_css();
    }

    fn activate(&self) {
        self.parent_activate();

        let has_socket = unsafe { self.obj().data::<String>(SOCKET_ID_KEY).is_some() };

        if has_socket {
            self.connect_endpoint();
        } else {
            self.show_ui();
        }
    }
}
impl GtkApplicationImpl for TasDiApp {}
