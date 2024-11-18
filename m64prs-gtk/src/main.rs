
mod controls;
mod ui;
mod logging;

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GTK_CSD", "0");

    logging::retarget_glib_logs();
    env_logger::init();
    ui::run_ui();
}