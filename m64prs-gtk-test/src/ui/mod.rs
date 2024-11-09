use std::env;
use std::sync::OnceLock;

use gtk::gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};
use send_wrapper::SendWrapper;

use crate::emu;

mod actions;
mod macros;
mod menu;

struct UIGlobals {
    application: Application,
    main_window: ApplicationWindow,
}

const APP_ID: &str = "io.github.jgcodes.m64prs";
static UI_GLOBALS: OnceLock<SendWrapper<UIGlobals>> = OnceLock::new();

pub fn run_ui() -> glib::ExitCode {
    let application = Application::new(Some(APP_ID), ApplicationFlags::FLAGS_NONE);

    {
        let self_path = env::current_exe().unwrap();
        let core_path = self_path.parent().unwrap().join("libmupen64plus.so");
        emu::init_core(&core_path, |_core| {}).unwrap();
    }

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

        UI_GLOBALS.get_or_init(|| {
            SendWrapper::new(UIGlobals {
                application: app.clone(),
                main_window: window.clone(),
            })
        });

        window.present();
    });

    application.run()
}
