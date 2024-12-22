use glib::{clone::Downgrade, translate::ToGlibPtr, SendWeakRef};
use gtk::prelude::*;

mod inner {
    use std::{
        cell::{OnceCell, RefCell},
        rc::Rc,
        sync::{Arc, Mutex},
    };

    use glib::SendWeakRef;
    use gtk::{prelude::*, subclass::prelude::*};
    use tasinput_protocol::{Endpoint, HostMessage, UiMessage, UiReply};

    use crate::app::{FLAG_SOCKET, SOCKET_ID_KEY};

    use super::ApplicationHoldSendRef;

    #[derive(Default)]
    pub struct TasDiApp {
        endpoint: RefCell<Option<Endpoint<UiMessage, HostMessage>>>,
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
            let options = command_line.options_dict();
            let socket = options.lookup::<String>(FLAG_SOCKET).unwrap();
            println!("server socket: {:?}", socket);

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

        fn activate(&self) {
            self.parent_activate();

            let mut endpoint = self.endpoint.borrow_mut();

            // extract the socket key from CLI
            let socket_id = unsafe { self.obj().steal_data::<String>(SOCKET_ID_KEY).unwrap() };

            let app_ref: SendWeakRef<_> = self.obj().downgrade().into();
            let app_hold: Arc<Mutex<Option<ApplicationHoldSendRef>>> = Arc::new(Mutex::new(Some(
                ApplicationHoldSendRef::new(self.obj().upcast_ref::<gio::Application>()),
            )));
            *endpoint = Some(
                Endpoint::<UiMessage, HostMessage>::connect(&socket_id, move |request| {
                    let app_ref = app_ref.clone();
                    let app_hold = app_hold.clone();
                    async move {
                        use tasinput_protocol::HostRequest::*;
                        match request {
                            Ping => {
                                println!("pong!");
                                UiReply::Ack
                            }
                            Close => {
                                let app_ref = app_ref.clone();
                                glib::spawn_future(async move {
                                    app_hold.lock().unwrap().take();
                                    if let Some(app) = app_ref.upgrade() {
                                        app.quit();
                                    }
                                });
                                UiReply::Ack
                            }
                            InitControllers { active } => UiReply::Ack,
                            SetVisibility { visible } => UiReply::Ack,
                        }
                    }
                })
                .unwrap(),
            );
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

struct ApplicationHoldSendRef(SendWeakRef<gio::Application>);

impl ApplicationHoldSendRef {
    fn new(app: &gio::Application) -> Self {
        unsafe {
            gio::ffi::g_application_hold(app.to_glib_none().0);
        }
        Self(glib::object::ObjectExt::downgrade(app).into())
    }
}

impl Drop for ApplicationHoldSendRef {
    fn drop(&mut self) {
        if let Some(app) = self.0.upgrade() {
            unsafe { gio::ffi::g_application_release(app.to_glib_none().0) }
        }
    }
}
