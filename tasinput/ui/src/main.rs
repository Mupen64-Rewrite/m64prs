use gtk::prelude::*;
use main_window::MainWindow;

mod enums;
mod joystick;
mod main_window;

fn main() {
    let app = gtk::Application::new(Some(APP_ID), gio::ApplicationFlags::NON_UNIQUE);

    app.add_main_option(
        "server-socket",
        glib::Char::from(b's'),
        glib::OptionFlags::NONE,
        glib::OptionArg::String,
        "ZeroMQ host to connect to.",
        None,
    );

    app.connect_command_line(on_command_line);
    app.connect_startup(on_startup);
    app.connect_activate(on_activate);
    app.run();
}

const APP_ID: &'static str = "io.github.jgcodes2020.tasdi";

fn on_command_line(app: &gtk::Application, cli: &gio::ApplicationCommandLine) -> i32 {
    let options = cli.options_dict();
    let socket = options.lookup::<Option<String>>("server-socket");
    println!("socket: {:?}", socket);
    -1
}
fn on_startup(_app: &gtk::Application) {
    const CSS_CONTENT: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/main.css"));

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(CSS_CONTENT);

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn on_activate(app: &gtk::Application) {
    MainWindow::setup_and_show(app);
}
