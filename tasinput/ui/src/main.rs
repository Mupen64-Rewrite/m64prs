use app::TasDiApp;
use gtk::prelude::*;

mod app;
mod enums;
mod joystick;
mod main_window;

fn main() {
    let app = TasDiApp::new();
    app.run();
}
