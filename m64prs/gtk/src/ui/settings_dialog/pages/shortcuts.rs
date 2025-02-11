use crate::ui::settings_dialog::SettingsPage;
mod inner {
    use gtk::subclass::prelude::*;

    use crate::ui::{
        core::CoreReadyState,
        settings_dialog::{settings_page::SettingsPageImpl, SettingsPage},
    };

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "shortcuts.ui")]
    pub struct ShortcutsPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for ShortcutsPage {
        const NAME: &'static str = "M64PRS_SettingsShortcutsPage";
        type Type = super::ShortcutsPage;
        type ParentType = gtk::Box;
        type Interfaces = (SettingsPage,);

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ShortcutsPage {}
    impl WidgetImpl for ShortcutsPage {}
    impl BoxImpl for ShortcutsPage {}

    impl SettingsPageImpl for ShortcutsPage {
        async fn load_page(&self, _state: &mut CoreReadyState) {}

        async fn save_page(&self, _state: &mut CoreReadyState) {}
    }
}

glib::wrapper! {
    pub struct ShortcutsPage(ObjectSubclass<inner::ShortcutsPage>)
        @extends
            gtk::Box,
            gtk::Widget,
        @implements
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget,
            SettingsPage;
}
