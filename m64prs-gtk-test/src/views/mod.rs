use std::env;
use std::sync::OnceLock;

use gtk::gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};
use send_wrapper::SendWrapper;

use crate::emu;

mod macros;
mod menu;

const APP_ID: &str = "io.github.jgcodes.m64prs";

pub fn run_ui() -> glib::ExitCode {
    let application = Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);

    {
        let self_path = env::current_exe().unwrap();
        let core_path = self_path.parent().unwrap().join("libmupen64plus.so");
        emu::init_core(&core_path, |_core| {}).unwrap();
    }

    application.connect_activate(|app| {


        let window = ApplicationWindow::builder()
            .application(app)
            .title("m64prs")
            .default_width(640)
            .default_height(480)
            .show_menubar(true)
            .build();

        window.present();
    });

    application.run()
}
