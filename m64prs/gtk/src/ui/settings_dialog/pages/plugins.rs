use crate::ui::settings_dialog::SettingsPage;
use gtk::{prelude::*, subclass::prelude::*};

mod utils;

mod inner {
    use std::{
        cell::Cell,
        ffi::OsStr,
        fs,
        path::{Path, PathBuf},
    };

    use gtk::{prelude::*, subclass::prelude::*};
    use m64prs_core::plugin::{AnyPlugin, PluginType};

    use crate::{
        ui::{
            core::CoreReadyState,
            settings_dialog::{parts::PluginSelect, settings_page::SettingsPageImpl, SettingsPage},
        },
        utils::paths::{is_shared_library, CONFIG_DIR, INSTALL_DIRS},
    };

    #[derive(glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::PluginsPage)]
    #[template(file = "plugins.ui")]
    pub struct PluginsPage {
        init: Cell<bool>,
        #[property(get)]
        graphics_plugins: gio::ListStore,
        #[property(get)]
        audio_plugins: gio::ListStore,
        #[property(get)]
        input_plugins: gio::ListStore,
        #[property(get)]
        rsp_plugins: gio::ListStore,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginsPage {
        const NAME: &'static str = "M64PRS_SettingsPluginsPage";
        type Type = super::PluginsPage;
        type ParentType = gtk::Box;
        type Interfaces = (SettingsPage,);

        fn class_init(class: &mut Self::Class) {
            PluginSelect::ensure_type();
            class.bind_template();
        }

        fn new() -> Self {
            Self {
                init: Cell::new(false),
                graphics_plugins: gio::ListStore::new::<gio::File>(),
                audio_plugins: gio::ListStore::new::<gio::File>(),
                input_plugins: gio::ListStore::new::<gio::File>(),
                rsp_plugins: gio::ListStore::new::<gio::File>(),
            }
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginsPage {}
    impl WidgetImpl for PluginsPage {}
    impl BoxImpl for PluginsPage {}

    impl SettingsPageImpl for PluginsPage {
        async fn load_page(&self, state: &mut CoreReadyState) {
            let plugin_dir = gio::File::for_path(&INSTALL_DIRS.plugin_dir);
            let e = plugin_dir
                .enumerate_children_future(
                    "",
                    gio::FileQueryInfoFlags::NONE,
                    glib::Priority::DEFAULT,
                )
                .await
                .unwrap();
            if self.init.replace(false) {
                self.graphics_plugins.remove_all();
                self.audio_plugins.remove_all();
                self.input_plugins.remove_all();
                self.rsp_plugins.remove_all();
            }

            while let Some(file) = e
                .next_files_future(1, glib::Priority::DEFAULT)
                .await
                .unwrap()
                .into_iter()
                .next()
            {
                // check that it's a valid shared library
                let file_name = file.name();
                if !is_shared_library(&file_name)
                {
                    continue;
                }
                println!("found {}", file_name.display());

                // try to open it as a plugin
                let plugin_path = INSTALL_DIRS.plugin_dir.join(&file_name);
                let (plugin_type, plugin_path) = gio::spawn_blocking(move || {
                    // We have to move the path into, then back out of the blocking task.
                    // The order matters; we have to borrow to load it, *then* move it,
                    // so it has to be returned second.
                    (
                        AnyPlugin::load(&plugin_path)
                            .ok()
                            .map(|plugin| plugin.plugin_type()),
                        plugin_path,
                    )
                })
                .await
                .unwrap();
                let plugin_type = match plugin_type {
                    Some(plugin_type) => plugin_type,
                    None => continue,
                };

                // sort it into the correct list
                let gio_plugin_path = gio::File::for_path(&plugin_path);
                match plugin_type {
                    PluginType::Graphics => self.graphics_plugins.append(&gio_plugin_path),
                    PluginType::Audio => self.audio_plugins.append(&gio_plugin_path),
                    PluginType::Input => self.input_plugins.append(&gio_plugin_path),
                    PluginType::Rsp => self.rsp_plugins.append(&gio_plugin_path),
                }
            }
            self.init.set(true);
        }

        async fn save_page(&self, state: &mut CoreReadyState) {}
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
