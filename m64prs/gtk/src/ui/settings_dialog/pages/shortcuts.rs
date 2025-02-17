use crate::ui::settings_dialog::SettingsPage;
mod inner {
    use std::{
        borrow::Cow,
        ffi::{CStr, CString},
        sync::LazyLock,
    };

    use glib::{
        object::{CastNone, ObjectExt},
        translate::IntoGlib,
        types::{StaticType, StaticTypeExt},
        GString,
    };
    use gtk::{prelude::*, subclass::prelude::*};
    use m64prs_core::Core;
    use m64prs_gtk_utils::glib_callback;
    use tr::tr;

    use crate::{
        controls,
        ui::{
            core::CoreReadyState,
            settings_dialog::{parts::AccelModel, settings_page::SettingsPageImpl, SettingsPage},
            AccelInputDialog,
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
            self.lv_settings
                .set_factory(Some(&init_row_factory(&*self.obj())));

            // populate model
            for (name, action) in &*ACTION_TABLE {
                self.rows.append(&AccelModel::new(&name, action));
            }
            self.lv_settings.set_model(Some(&self.rows_sel));

            // bind filter
            self.en_search
                .bind_property("text", &self.filter, "search")
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

            for model in self.rows.iter::<AccelModel>().map(|res| res.unwrap()) {
                let action = model.action();
                let action_cstr = CString::new(&*action).unwrap();
                let value_cstr = sect.get_cast::<CString>(&action_cstr).unwrap();
                let value = value_cstr.to_string_lossy();

                model.set_accel(
                    (!value.is_empty()).then(|| gtk::accelerator_parse(&*value).unwrap()),
                );
            }
        }

        async fn save_page(&self, state: &mut CoreReadyState) {
            let mut sect = state
                .cfg_open_mut(CFG_SECTION_KEY)
                .expect("Failed to open config section");

            for model in self.rows.iter::<AccelModel>().map(|res| res.unwrap()) {
                let action = model.action();
                let action_cstr = CString::new(&*action).unwrap();
                let value_gstr = match model.get_accel() {
                    Some((key, modifiers)) => Cow::Owned(gtk::accelerator_name(key, modifiers)),
                    None => Cow::Borrowed(glib::gstr!("")),
                };
                sect.set(&action_cstr, value_gstr.to_cstr_until_nul())
                    .unwrap();
            }
        }
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

    fn init_row_factory(page: &super::ShortcutsPage) -> gtk::ListItemFactory {
        static ROW_TEMPLATE: LazyLock<glib::Bytes> =
            LazyLock::new(|| glib::Bytes::from_static(include_bytes!("shortcuts_row.ui")));

        let page_ref = page.downgrade();

        let scope = gtk::BuilderRustScope::new();
        scope.add_callback("click_handler", {
            let page_ref = page_ref.clone();
            glib_callback!(move |this: &gtk::ListItem,
                                 n_press: i32,
                                 _x: f64,
                                 _y: f64,
                                 _: gtk::GestureClick| {
                let page = page_ref.upgrade().unwrap();
                if n_press == 2 {
                    let this = this.clone();
                    glib::spawn_future_local(async move {
                        println!("yeet");
                        let window = page.root().and_downcast::<gtk::Window>().unwrap();
                        let dialog = AccelInputDialog::new();
                        let (key, modifiers) = match dialog.prompt(Some(&window)).await {
                            Some(result) => result,
                            None => return,
                        };

                        let model = this.item().and_downcast::<AccelModel>().unwrap();
                        model.set_key(0);
                        model.set_modifiers(gdk::ModifierType::empty());

                        if key == 0 {
                            return;
                        }

                        let found_conflict = page
                            .imp()
                            .rows
                            .iter::<AccelModel>()
                            .map(|res| res.unwrap())
                            .any(|node| node.key() == key && node.modifiers() == modifiers);

                        if found_conflict {
                            // error dialog? Potential override?
                            return;
                        }

                        model.set_key(key);
                        model.set_modifiers(modifiers);
                    });
                }
            })
        });
        let row_factory = gtk::BuilderListItemFactory::from_bytes(Some(&scope), &ROW_TEMPLATE);
        row_factory.upcast()
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
