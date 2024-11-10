use glib::object::ObjectExt;
use gtk::{prelude::*};

use crate::view_models;

pub fn build_view(app: &gtk::Application) -> (gtk::ApplicationWindow, view_models::MainViewModel) {
    let view_model = view_models::MainViewModel::new();

    relm4_macros::view! {
        view_root = gtk::Stack {},
        window = gtk::ApplicationWindow::new(app) {
            set_child: Some(&view_root)
        }
    }

    window.bind_property("title", &view_model, "title").build();
    window.bind_property("resizable", &view_model, "resizable").build();

    (window, view_model)
}