mod settings_page;
mod pages;
mod parts;

pub(self) use settings_page::SettingsPage;

mod inner {
    use gtk::{prelude::*, subclass::prelude::*};

    use super::pages::{self, EmuPage};

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "mod.ui")]
    pub struct SettingsDialog {
        #[template_child]
        tabs_nb: TemplateChild<gtk::Notebook>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsDialog {
        const NAME: &'static str = "M64PRS_SettingsDialog";
        type Type = super::SettingsDialog;
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            pages::init_pages();
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