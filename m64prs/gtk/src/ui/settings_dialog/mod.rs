mod inner {
    use gtk::{prelude::*, subclass::prelude::*};

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/ui/settings_dialog/mod.blp")]
    pub struct SettingsDialog {

    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsDialog {
        const NAME: &'static str = "M64PRS_SettingsDialog";
        type Type = super::SettingsDialog;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for SettingsDialog {}
    impl WidgetImpl for SettingsDialog {}
    impl WindowImpl for SettingsDialog {}
}

glib::wrapper! {
    pub struct SettingsDialog(ObjectSubclass<inner::SettingsDialog>)
        @extends 
            gtk::Window, 
            gtk::Widget,
        @implements 
            gtk::Accessible, 
            gtk::Buildable, 
            gtk::ConstraintTarget, 
            gtk::Native, 
            gtk::Root;
}

impl SettingsDialog {
    pub fn new() -> SettingsDialog {
        glib::Object::new::<SettingsDialog>()
    }
}