use crate::ui::settings_dialog::SettingsPage;
use gtk::{prelude::*, subclass::prelude::*};

mod inner {
    use std::{cell::Cell, ffi::{c_int, CStr}};

    use gtk::{prelude::*, subclass::prelude::*};
    use m64prs_sys::common::ConfigValue;

    use crate::ui::{
        core::CoreReadyState,
        settings_dialog::{settings_page::SettingsPageImpl, SettingsPage},
    };

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "emu.ui")]
    #[properties(wrapper_type = super::EmuPage)]
    pub struct EmuPage {
        #[property(get, set, default = 2)]
        r4300_emulator: Cell<u32>,
        #[property(get, set, default = true)]
        randomize_interrupt: Cell<bool>,
        #[property(get, set, default = false)]
        disable_expansion_pak: Cell<bool>,
    }

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

    #[glib::derived_properties]
    impl ObjectImpl for EmuPage {}
    impl WidgetImpl for EmuPage {}
    impl BoxImpl for EmuPage {}

    const SECTION_NAME: &CStr = c"Core";

    impl SettingsPageImpl for EmuPage {
        async fn load_page(&self, state: &mut CoreReadyState) {
            // TODO: proper error messages and whatnot
            let sect = state.cfg_open_mut(SECTION_NAME).expect("Failed to open config section");
            let this = self.obj();

            println!("hello");
            
            this.set_r4300_emulator(sect.get_cast_or(2, c"R4300Emulator").unwrap() as u32);
            this.set_randomize_interrupt(sect.get_cast_or(true, c"RandomizeInterrupt").unwrap());
            this.set_disable_expansion_pak(sect.get_cast_or(false, c"DisableExtraMem").unwrap());

        }

        async fn save_page(&self, state: &mut CoreReadyState) {
            // TODO: proper error messages and whatnot
            let mut sect = state.cfg_open_mut(SECTION_NAME).expect("Failed to open config section");
            let this = self.obj();

            sect.set(c"R4300Emulator", this.r4300_emulator() as i32).unwrap();
            sect.set(c"RandomizeInterrupt", this.randomize_interrupt()).unwrap();
            sect.set(c"DisableExtraMem", this.disable_expansion_pak()).unwrap();

            sect.save().unwrap();
        }
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
