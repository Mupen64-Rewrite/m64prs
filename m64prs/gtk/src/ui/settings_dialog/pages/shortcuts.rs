use crate::ui::settings_dialog::SettingsPage;
mod inner {
    use std::{
        ffi::{CStr, CString},
        sync::LazyLock,
    };

    use glib::{object::{CastNone, ObjectExt}, types::{StaticType, StaticTypeExt}};
    use gtk::{subclass::prelude::*, prelude::*};
    use m64prs_core::Core;
    use m64prs_gtk_utils::glib_callback;
    use tr::tr;

    use crate::{
        controls,
        ui::{
            core::CoreReadyState,
            settings_dialog::{parts::AccelModel, settings_page::SettingsPageImpl, SettingsPage},
        },
    };

    static ACTION_TABLE: LazyLock<[(String, &'static str); 15]> = LazyLock::new(|| {
        [
            (tr!("main_act" => "Open ROM"), "app.file.open_rom"),
            (tr!("main_act" => "Close ROM"), "app.file.close_rom"),
            (tr!("main_act" => "Settings"), "app.file.settings"),
            (tr!("main_act" => "Pause/Resume"), "app.emu.toggle_pause"),
            (tr!("main_act" => "Frame Advance"), "app.emu.frame_advance"),
            (tr!("main_act" => "Reset ROM"), "app.emu.reset_rom"),
            (tr!("main_act" => "Save State"), "app.emu.save_state"),
            (tr!("main_act" => "Load State"), "app.emu.load_state"),
            (tr!("main_act" => "Save State to..."), "app.emu.save_file"),
            (tr!("main_act" => "Load State from..."), "app.emu.load_file"),
            (tr!("main_act" => "New Movie"), "app.vcr.new_movie"),
            (tr!("main_act" => "Load Movie"), "app.vcr.load_movie"),
            (tr!("main_act" => "Save Movie"), "app.vcr.save_movie"),
            (tr!("main_act" => "Close Movie"), "app.vcr.close_movie"),
            (
                tr!("main_act" => "Read-only Mode"),
                "app.vcr.toggle_read_only",
            ),
        ]
    });

    #[derive(gtk::CompositeTemplate)]
    #[template(file = "shortcuts.ui")]
    pub struct ShortcutsPage {
        #[template_child]
        en_search: TemplateChild<gtk::Entry>,
        #[template_child]
        lv_settings: TemplateChild<gtk::ListView>,
        filter: gtk::StringFilter,
        rows: gio::ListStore,
        rows_sel: gtk::SingleSelection,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShortcutsPage {
        const NAME: &'static str = "M64PRS_SettingsShortcutsPage";
        type Type = super::ShortcutsPage;
        type ParentType = gtk::Box;
        type Interfaces = (SettingsPage,);

        fn new() -> Self {
            let filter = gtk::StringFilter::new(Some(gtk::PropertyExpression::new(
                AccelModel::static_type(),
                None::<gtk::Expression>,
                "name",
            )));
            filter.set_ignore_case(true);
            filter.set_match_mode(gtk::StringFilterMatchMode::Substring);

            let rows = gio::ListStore::new::<AccelModel>();
            let rows_filt = gtk::FilterListModel::new(Some(rows.clone()), Some(filter.clone()));
            let rows_sel = gtk::SingleSelection::new(Some(rows_filt));

            Self {
                en_search: Default::default(),
                lv_settings: Default::default(),
                filter,
                rows,
                rows_sel,
            }
        }

        fn class_init(class: &mut Self::Class) {
            AccelModel::ensure_type();
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // #[glib::derived_properties]
    impl ObjectImpl for ShortcutsPage {
        fn constructed(&self) {
            static ROW_TEMPLATE: LazyLock<glib::Bytes> =
                LazyLock::new(|| glib::Bytes::from_static(include_bytes!("shortcuts_row.ui")));

            // setup row display
            let scope = gtk::BuilderRustScope::new();
            scope.add_callback("click_handler", glib_callback!(|
                this: &gtk::ListItem, n_press: i32, x: f64, y: f64, _: gtk::GestureClick
            | {
                if n_press == 2 {
                    println!("double click: {}", this.item().and_downcast::<AccelModel>().unwrap().name())
                }
            }));
            let row_factory =
                gtk::BuilderListItemFactory::from_bytes(Some(&scope), &ROW_TEMPLATE);
            self.lv_settings.set_factory(Some(&row_factory));

            // populate model
            for (name, action) in &*ACTION_TABLE {
                self.rows.append(&AccelModel::new(&name, action));
            }
            self.lv_settings.set_model(Some(&self.rows_sel));

            // bind filter
            self.en_search.bind_property("text", &self.filter, "search")
                .sync_create()
                .build();

        }
    }
    impl WidgetImpl for ShortcutsPage {}
    impl BoxImpl for ShortcutsPage {}

    const CFG_SECTION_KEY: &CStr = c"M64PRS-Shortcuts";

    impl SettingsPageImpl for ShortcutsPage {
        async fn load_page(&self, state: &mut CoreReadyState) {
            let sect = state
                .cfg_open_mut(CFG_SECTION_KEY)
                .expect("Failed to open config section");

            // TODO
        }

        async fn save_page(&self, _state: &mut CoreReadyState) {}
    }

    /// Set the default config for this page.
    pub(in super::super) fn default_config(core: &mut Core) {
        let mut sect = core
            .cfg_open_mut(CFG_SECTION_KEY)
            .expect("Failed to open config section");
        for (_, action) in &*ACTION_TABLE {
            let action_cstr = CString::new(action.as_bytes()).unwrap();
            sect.set_default(&action_cstr, c"", c"action help (auto-generated)")
                .unwrap();
        }
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

pub(super) use inner::default_config;
