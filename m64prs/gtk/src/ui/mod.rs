use gio::ApplicationFlags;
use gtk::prelude::*;

mod accel_input_dialog;
mod core;
mod main_window;
mod movie_dialog;
mod settings_dialog;

use main_window::MainWindow;
use movie_dialog::MovieDialog;
use settings_dialog::SettingsDialog;
use accel_input_dialog::AccelInputDialog;

const APP_ID: &str = "io.github.jgcodes.m64prs";

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
struct AppDialogError(String);

pub fn run_ui() {
    // this catches some template errors early
    MainWindow::ensure_type();
    MovieDialog::ensure_type();
    SettingsDialog::ensure_type();

    let app = gtk::Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);
    app.connect_activate(|app| MainWindow::setup_and_show(app));

    app.set_accels_for_action("app.file.open_rom", &["<Ctrl>o"]);

    app.run();
}
