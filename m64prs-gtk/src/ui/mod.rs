use gio::ApplicationFlags;
use gtk::prelude::*;

mod main_window;
mod core;

use main_window::MainWindow;

const APP_ID: &str = "io.github.jgcodes.m64prs";

pub fn run_ui() {
    let app = gtk::Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);
    app.connect_activate(|app| MainWindow::setup_and_show(app));
    app.run();
}
