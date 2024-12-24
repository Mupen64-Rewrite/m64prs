use gtk::prelude::*;

mod inner {
    use std::{cell::RefCell, mem};

    use glib::SendWeakRef;
    use gtk::{prelude::*, subclass::prelude::*};
    use m64prs_sys::Buttons;
    use tasinput_protocol::{Endpoint, HostMessage, UiMessage, UiReply, PING_TIMEOUT};
    use tokio::{
        sync::mpsc::{self, error::TryRecvError},
        time::{sleep_until, Instant},
    };

    use crate::{
        app::{FLAG_SOCKET, SOCKET_ID_KEY},
        main_window::MainWindow,
    };

    #[derive(Default)]
    pub struct TasDiApp {
        endpoint: RefCell<Option<Endpoint<UiMessage, HostMessage>>>,
        windows: RefCell<[Option<MainWindow>; 4]>,
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
                let (timeout_send, mut timeout_recv) = mpsc::channel::<Instant>(16);

                // Main endpoint request handler.
                let mut endpoint = Endpoint::<UiMessage, HostMessage>::connect(&socket_id, {
                    // These need to be cloned so that the FnMut can hand out copies.
                    let app_ref = app_ref.clone();
                    let timeout_send = timeout_send.clone();
                    move |request| {
                        // These need to be cloned so that each future gets a copy.
                        let app_ref = app_ref.clone();
                        let timeout_send = timeout_send.clone();
                        async move {
                            use tasinput_protocol::HostRequest::*;
                            match request {
                                Ping => {
                                    timeout_send
                                        .try_send(Instant::now() + PING_TIMEOUT)
                                        .unwrap();
                                    UiReply::Ack
                                }
                                Close => {
                                    let app_ref = app_ref.clone();
                                    glib::spawn_future(async move {
                                        if let Some(app) = app_ref.upgrade() {
                                            app.quit();
                                        }
                                    });
                                    UiReply::Ack
                                }
                                InitControllers { active } => {
                                    let app_ref = app_ref.clone();
                                    glib::spawn_future(async move {
                                        if let Some(app) = app_ref.upgrade() {
                                            let bits = active.bits();
                                            let mut windows = app.imp().windows.borrow_mut();
                                            for i in 0..4 {
                                                if (bits & (1 << i)) != 0 {
                                                    windows[i] = Some(MainWindow::new(&app))
                                                }
                                            }
                                        }
                                    });
                                    UiReply::Ack
                                }
                                SetVisible { visible } => {
                                    let app_ref = app_ref.clone();
                                    glib::spawn_future(async move {
                                        if let Some(app) = app_ref.upgrade() {
                                            let windows = app.imp().windows.borrow();
                                            for window in &*windows {
                                                window
                                                    .as_ref()
                                                    .inspect(|&window| window.set_visible(visible));
                                            }
                                        }
                                    });
                                    UiReply::Ack
                                }
                                PollState { controller } => {
                                    let app_ref = app_ref.clone();
                                    let buttons = glib::spawn_future(async move {
                                        if let Some(app) = app_ref.upgrade() {
                                            let windows = app.imp().windows.borrow();
                                            windows[controller as usize]
                                                .as_ref()
                                                .map_or(Buttons::BLANK, |val| val.to_buttons())
                                        } else {
                                            Buttons::BLANK
                                        }
                                    })
                                    .await
                                    .unwrap();

                                    UiReply::PolledState { buttons }
                                }
                            }
                        }
                    }
                })
                .unwrap();

                // Ping timeout handler.
                endpoint.spawn(|handle| {
                    let app_ref = app_ref.clone();
                    async move {
                        let cancel_token = handle.cancel_token();
                        // Wait for the first ping.
                        let mut next_time = tokio::select! {
                            biased;
                            _ = cancel_token.cancelled() => return,
                            first_time = timeout_recv.recv() => first_time.unwrap()
                        };
                        // Ensure that any subsequent pings occur before the timeout.
                        loop {
                            tokio::select! {
                                biased;
                                _ = cancel_token.cancelled() => return,
                                _ = sleep_until(next_time) => (),
                            }
                            match timeout_recv.try_recv() {
                                Ok(time) => {
                                    next_time = time;
                                }
                                Err(TryRecvError::Empty) => {
                                    let app_ref = app_ref.clone();
                                    glib::spawn_future(async move {
                                        if let Some(app) = app_ref.upgrade() {
                                            app.quit();
                                        }
                                    });
                                }
                                Err(TryRecvError::Disconnected) => {
                                    panic!("The sender should not disconnect.")
                                }
                            }
                        }
                    }
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
}

const APP_ID: &'static str = "io.github.jgcodes2020.tasdi";
const FLAG_SOCKET: &'static str = "server-socket";
const SOCKET_ID_KEY: &'static str = "io.github.jgcodes2020.tasdi.socket_id";

glib::wrapper! {
    pub struct TasDiApp(ObjectSubclass<inner::TasDiApp>)
        @extends
            gtk::Application,
            gio::Application,
        @implements
            gio::ActionGroup,
            gio::ActionMap;
}

impl TasDiApp {
    pub fn new() -> Self {
        gtk::init().unwrap();

        let props: &mut [(&str, glib::Value)] = &mut [
            ("application-id", Some(APP_ID).to_value()),
            (
                "flags",
                (gio::ApplicationFlags::NON_UNIQUE | gio::ApplicationFlags::HANDLES_COMMAND_LINE)
                    .to_value(),
            ),
        ];

        let result: Self =
            unsafe { glib::Object::with_mut_values(Self::static_type(), props).unsafe_cast() };

        result.add_main_option(
            FLAG_SOCKET,
            b's'.into(),
            glib::OptionFlags::NONE,
            glib::OptionArg::String,
            "Plugin socket server to connect.",
            None,
        );

        result
    }
}
