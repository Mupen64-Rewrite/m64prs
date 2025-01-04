mod settings_page;
mod pages;

pub(self) use settings_page::SettingsPage;

mod inner {
    use gtk::{prelude::*, subclass::prelude::*};

    use super::pages::EmuPage;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/ui/settings_dialog/mod.blp")]
    pub struct SettingsDialog {

    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsDialog {
        const NAME: &'static str = "M64PRS_SettingsDialog";
        type Type = super::SettingsDialog;
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            EmuPage::ensure_type();
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
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