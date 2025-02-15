mod controls;
mod i18n;
mod logging;
mod ui;
mod utils;

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GTK_CSD", "0");

    i18n::setup_gettext();
    logging::retarget_glib_logs();

    env_logger::init_from_env(env_logger::Env::new().filter_or("RUST_LOG", "warn"));
    ui::run_ui();
}
