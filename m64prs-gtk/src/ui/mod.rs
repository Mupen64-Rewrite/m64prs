use gio::ApplicationFlags;
use gtk::prelude::*;

mod core;
mod main_window;
mod movie_dialog;

use main_window::MainWindow;
use movie_dialog::MovieDialog;

const APP_ID: &str = "io.github.jgcodes.m64prs";

pub fn run_ui() {
    // this catches some template errors early
    MainWindow::ensure_type();
    MovieDialog::ensure_type();

    let app = gtk::Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);
    app.connect_activate(|app| MainWindow::setup_and_show(app));
    app.run();
}
