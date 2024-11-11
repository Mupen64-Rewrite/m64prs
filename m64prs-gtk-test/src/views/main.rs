use glib::object::ObjectExt;
use gtk::prelude::*;

use crate::view_models;

use super::macros;

pub fn build_view(app: &gtk::Application) -> gtk::ApplicationWindow {
    let view_model = view_models::MainViewModel::new();

    // relm4_macros::view! {
    //     view_root = gtk::Stack {
    //         gtk::Widget {

    //         }
    //     },
    //     window = gtk::ApplicationWindow::new(app) {
    //         set_child: Some(&view_root)
    //     }
    // }

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .child(&{
            let stack = gtk::Stack::new();

            stack.set_size_request(640, 480);
            stack
        })
        .default_width(-1)
        .default_height(-1)
        .build();

    window.bind_property("title", &view_model, "title").build();
    window
        .bind_property("resizable", &view_model, "resizable")
        .build();

    // tie lifetime of view model to window
    macros::take_owner!(view_model -> window);

    window
}
