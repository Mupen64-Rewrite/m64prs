mod controls;
mod i18n;
mod logging;
// mod old_ui;
mod ui;
mod utils;

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GTK_CSD", "0");

    i18n::setup_gettext();
    logging::retarget_glib_logs();
    env_logger::init();
    ui::run_ui();
}
