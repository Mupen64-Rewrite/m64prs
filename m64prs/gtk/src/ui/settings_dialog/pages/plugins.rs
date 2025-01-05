use gtk::{prelude::*, subclass::prelude::*};
use crate::ui::settings_dialog::SettingsPage;

mod inner {
    use gtk::{prelude::*, subclass::prelude::*};

    use crate::ui::{core::CoreReadyState, settings_dialog::{settings_page::SettingsPageImpl, SettingsPage}};

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "plugins.ui")]
    pub struct PluginsPage {

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginsPage {
        const NAME: &'static str = "M64PRS_SettingsPluginsPage";
        type Type = super::PluginsPage;
        type ParentType = gtk::Box;
        type Interfaces = (SettingsPage,);

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginsPage {}
    impl WidgetImpl for PluginsPage {}
    impl BoxImpl for PluginsPage {}

    impl SettingsPageImpl for PluginsPage {
        fn load_from_core(&self, state: &mut CoreReadyState) {
            
        }
    
        fn save_to_core(&self, state: &mut CoreReadyState) {
            
        }
    }
}

glib::wrapper! {
    pub struct PluginsPage(ObjectSubclass<inner::PluginsPage>)
        @extends
            gtk::Box,
            gtk::Widget,
        @implements
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget,
            SettingsPage;
}

