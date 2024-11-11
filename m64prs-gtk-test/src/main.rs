use glib::{LogField, LogLevel, LogWriterOutput};
use gtk::glib;

mod controls;
mod view_models;
mod views;

fn main() {
    glib::log_set_default_handler(glib::rust_log_handler);

    env_logger::init();
    views::run_ui();
}
