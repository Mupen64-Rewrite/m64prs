use relm4::RelmApp;

mod actions;
mod core;
mod file_dialog;
mod main;

pub(crate) fn run_ui() {
    let app = RelmApp::new("io.github.jgcodes.m64prs");
    app.run::<main::Model>(());
}