mod enums;
mod window;

mod inner {
    use std::cell::Cell;
    use std::fs;
    use std::path::PathBuf;

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use m64prs_vcr::movie::M64Header;

    use super::window::MovieDialogWindow;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MovieDialog)]
    pub struct MovieDialog {
        #[property(get, set, default_value = false)]
        load: Cell<bool>,
    }

    #[m64prs_gtk_utils::forward_wrapper(super::MovieDialog, vis = pub)]
    impl MovieDialog {
        pub(super) async fn new_movie(
            &self,
            transient_for: Option<&impl IsA<gtk::Window>>,
        ) -> Option<(PathBuf, M64Header)> {
            let window = MovieDialogWindow::with_load(false);
            if !window.prompt(transient_for).await {
                return None;
            };

            let mut header = M64Header::default();
            header.start_flags = window.start_type().into();
            header.author.write_clipped(&window.author());
            header.description.write_clipped(&window.description());

            let path = window
                .cur_file()
                .as_ref()
                .and_then(FileExt::path)
                .expect("file with no path?? impossible!");

            Some((path, header))
        }

        pub(super) async fn load_movie(
            &self,
            transient_for: Option<&impl IsA<gtk::Window>>,
        ) -> Option<gio::File> {
            let window = MovieDialogWindow::with_load(true);
            if !window.prompt(transient_for).await {
                return None;
            };

            let path = window.cur_file().expect("no file??");

            Some(path)
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
