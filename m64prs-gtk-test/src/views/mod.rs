use std::env;
use std::sync::OnceLock;

use gtk::gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};
use send_wrapper::SendWrapper;

mod macros;
mod menu;
mod main;

const APP_ID: &str = "io.github.jgcodes.m64prs";

pub fn run_ui() -> glib::ExitCode {
    let application = Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("m64prs")
            .show_menubar(true)
            .build();

        window.present();
    });

    application.run()
}
