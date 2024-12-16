use gio::ApplicationFlags;
use gtk::prelude::*;
use main_window::MainWindow;

mod enums;
mod joystick;
mod main_window;

const APP_ID: &str = "io.github.jgcodes.tasdi";

fn load_css() {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_bytes(&glib::Bytes::from_static(include_bytes!("main.css")));

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    let app = gtk::Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);
    app.connect_startup(|app| load_css());
    app.connect_activate(|app| MainWindow::setup_and_show(app));
    app.run();
}
