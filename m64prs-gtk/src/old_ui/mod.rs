use relm4::RelmApp;

mod actions;
mod dialogs;
mod core;
mod main;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl.gen.rs"));
}

pub(crate) fn run_ui() {
    let app = RelmApp::new("io.github.jgcodes.m64prs");
    app.run::<main::Model>(());
}
