use crate::ui::settings_dialog::SettingsPage;
use gtk::{prelude::*, subclass::prelude::*};

mod inner {
    use gtk::{prelude::*, subclass::prelude::*};

    use crate::ui::{
        core::CoreReadyState,
        settings_dialog::{settings_page::SettingsPageImpl, SettingsPage},
    };

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "emu.ui")]
    pub struct EmuPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for EmuPage {
        const NAME: &'static str = "M64PRS_SettingsEmuPage";
        type Type = super::EmuPage;
        type ParentType = gtk::Box;
        type Interfaces = (SettingsPage,);

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EmuPage {}
    impl WidgetImpl for EmuPage {}
    impl BoxImpl for EmuPage {}

    impl SettingsPageImpl for EmuPage {
        fn load_from_core(&self, state: &mut CoreReadyState) {}

        fn save_to_core(&self, state: &mut CoreReadyState) {}
    }
}

glib::wrapper! {
    pub struct EmuPage(ObjectSubclass<inner::EmuPage>)
        @extends
            gtk::Box,
            gtk::Widget,
        @implements
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget,
            SettingsPage;
}
