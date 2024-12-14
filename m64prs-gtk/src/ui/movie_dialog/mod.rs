mod actions;
mod window;

mod inner {
    use std::cell::Cell;

    use gtk::subclass::prelude::*;
    use gtk::prelude::*;

    use super::window::MovieDialogWindow;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MovieDialog)]
    pub struct MovieDialog {
        #[property(get, set, default_value = false)]
        load: Cell<bool>,
    }

    #[m64prs_gtk_utils::forward_wrapper(super::MovieDialog, vis = pub)]
    impl MovieDialog {
        pub(super) async fn select_movie(&self, transient_for: Option<&impl IsA<gtk::Window>>) {
            let window = MovieDialogWindow::with_settings(&*self.obj());
            window.prompt(transient_for).await;
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MovieDialog {
        const NAME: &'static str = "M64PRS_MovieDialog";
        type Type = super::MovieDialog;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MovieDialog {}
}

glib::wrapper! {
    pub struct MovieDialog(ObjectSubclass<inner::MovieDialog>);
}