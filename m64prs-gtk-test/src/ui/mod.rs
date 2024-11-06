use std::sync::OnceLock;

use gtk::gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};
use send_wrapper::SendWrapper;

mod actions;
mod menu;
mod macros;

struct GlobalState {
    application: SendWrapper<Application>,
    main_window: SendWrapper<ApplicationWindow>,
}

const APP_ID: &str = "io.github.jgcodes.m64prs";
static GLOBAL_STATE: OnceLock<GlobalState> = OnceLock::new();

pub fn run_ui() -> glib::ExitCode {
    let application = Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);

    application.connect_activate(|app| {
        actions::register_actions(app);
        app.set_menubar(Some(&menu::create_menu()));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("m64prs")
            .default_width(640)
            .default_height(480)
            .show_menubar(true)
            .build();

        GLOBAL_STATE.get_or_init(|| GlobalState {
            application: SendWrapper::new(app.clone()),
            main_window: SendWrapper::new(window.clone()),
        });

        window.present();
    });

    application.run()
}
