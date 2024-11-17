
mod controls;
mod ui;
mod logging;

fn main() {
    logging::retarget_glib_logs();
    env_logger::init();
    ui::run_ui();
}