use gio::ApplicationFlags;
use gtk::{prelude::*};

mod main_window;
mod menu;

use main_window::MainWindow;

const APP_ID: &str = "io.github.jgcodes.m64prs";

pub fn run_ui() {
    let app = gtk::Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);
    app.connect_activate(|app| {
        let main_window = gtk::ApplicationWindow::new(app);
        main_window.present();
    });
    app.run();
}