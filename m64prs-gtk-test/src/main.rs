use gtk::glib;

mod controls;
mod view_models;
mod views;

fn main() {
    env_logger::init();
    glib::log_set_default_handler(glib::rust_log_handler);
    views::run_ui();
}
