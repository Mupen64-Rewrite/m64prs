use gtk::glib;

mod controls;
mod emu;
mod ui;

fn main() {
    env_logger::init();
    glib::log_set_default_handler(glib::rust_log_handler);
    ui::run_ui();
}
