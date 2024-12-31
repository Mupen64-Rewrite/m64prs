use gtk::prelude::*;

mod endpoint;
mod inner;

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
