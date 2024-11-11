mod controls;
mod view_models;
mod views;
mod logging;

fn main() {
    logging::retarget_glib_logs();
    env_logger::init();
    views::run_ui();
}