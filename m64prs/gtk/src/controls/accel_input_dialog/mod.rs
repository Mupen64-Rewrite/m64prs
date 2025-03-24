mod window;

mod inner {

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use super::window::AccelInputDialogWindow;

    #[derive(Default)]
    pub struct AccelInputDialog {}

    #[m64prs_gtk_utils::forward_wrapper(super::AccelInputDialog, vis = pub)]
    impl AccelInputDialog {
        pub(super) async fn prompt(
            &self,
            transient_for: Option<&impl IsA<gtk::Window>>,
        ) -> Option<(u32, gdk::ModifierType)> {
            let window = AccelInputDialogWindow::new();
            if !window.prompt(transient_for).await {
                return None;
            }
            let key = window.key();
            let mods = window.modifiers();
            Some((key, mods))
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccelInputDialog {
        const NAME: &'static str = "M64PRS_AccelInputDialog";
        type Type = super::AccelInputDialog;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AccelInputDialog {}
}

glib::wrapper! {
    pub struct AccelInputDialog(ObjectSubclass<inner::AccelInputDialog>);
}

impl AccelInputDialog {
    pub fn new() -> AccelInputDialog {
        glib::Object::new()
    }
}
