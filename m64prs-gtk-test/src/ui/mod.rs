use relm4::RelmApp;

mod actions;
mod main;
mod core_worker;

pub(crate) fn run_ui() {
    let app = RelmApp::new("io.github.jgcodes.m64prs");
    app.run::<main::Model>(());
}