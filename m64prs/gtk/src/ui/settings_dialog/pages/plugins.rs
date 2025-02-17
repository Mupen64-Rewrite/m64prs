use std::ffi::CString;

use m64prs_core::Core;

use crate::{ui::settings_dialog::SettingsPage, utils::paths::add_lib_ext};

mod inner {
    use core::str;
    use std::{
        cell::Cell,
        ffi::{CStr, CString},
    };

    use gtk::{prelude::*, subclass::prelude::*};
    use m64prs_core::{
        plugin::{AnyPlugin, PluginType},
        ConfigSectionMut,
    };

    use crate::{
        ui::{
            core::CoreReadyState,
            settings_dialog::{parts::PluginSelect, settings_page::SettingsPageImpl, SettingsPage},
        },
        utils::paths::{is_shared_library, INSTALL_DIRS},
    };

    const CFG_SECTION_KEY: &CStr = c"M64PRS-Plugins";
    const CFG_GRAPHICS: &CStr = c"Graphics";
    const CFG_AUDIO: &CStr = c"Audio";
    const CFG_INPUT: &CStr = c"Input";
    const CFG_RSP: &CStr = c"RSP";

    #[derive(glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::PluginsPage)]
    #[template(file = "plugins.ui")]
    pub struct PluginsPage {
        #[template_child]
        dd_graphics_plugins: gtk::TemplateChild<PluginSelect>,
        #[template_child]
        dd_audio_plugins: gtk::TemplateChild<PluginSelect>,
        #[template_child]
        dd_input_plugins: gtk::TemplateChild<PluginSelect>,
        #[template_child]
        dd_rsp_plugins: gtk::TemplateChild<PluginSelect>,
        #[property(get)]
        graphics_plugins: gio::ListStore,
        #[property(get)]
        audio_plugins: gio::ListStore,
        #[property(get)]
        input_plugins: gio::ListStore,
        #[property(get)]
        rsp_plugins: gio::ListStore,
        init: Cell<bool>,
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
                dd_graphics_plugins: Default::default(),
                dd_audio_plugins: Default::default(),
                dd_input_plugins: Default::default(),
                dd_rsp_plugins: Default::default(),
                graphics_plugins: gio::ListStore::new::<gio::File>(),
                audio_plugins: gio::ListStore::new::<gio::File>(),
                input_plugins: gio::ListStore::new::<gio::File>(),
                rsp_plugins: gio::ListStore::new::<gio::File>(),
                init: Cell::new(false),
            }
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginsPage {
        fn constructed(&self) {
            self.dd_graphics_plugins
                .set_plugins(self.graphics_plugins.clone());
            self.dd_audio_plugins
                .set_plugins(self.audio_plugins.clone());
            self.dd_input_plugins
                .set_plugins(self.input_plugins.clone());
            self.dd_rsp_plugins.set_plugins(self.rsp_plugins.clone());
        }
    }
    impl WidgetImpl for PluginsPage {}
    impl BoxImpl for PluginsPage {}

    impl PluginsPage {
        async fn check_plugins(&self) {
            let plugin_dir = gio::File::for_path(&INSTALL_DIRS.plugin_dir);
            let plugin_dir_iter = plugin_dir
                .enumerate_children_future(
                    "standard::name",
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

            while let Some(file) = plugin_dir_iter
                .next_files_future(1, glib::Priority::DEFAULT)
                .await
                .unwrap()
                .into_iter()
                .next()
            {
                // check that it's a valid shared library
                let file_name = plugin_dir_iter.child(&file).basename().unwrap();
                if !is_shared_library(&file_name) {
                    continue;
                }

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

        fn load_value_one(
            model: &gio::ListStore,
            sect: &mut ConfigSectionMut<'_>,
            key: &CStr,
            select: &PluginSelect,
        ) {
            let name_str = sect
                .get_cast::<CString>(key)
                .unwrap_or_else(|_| c"".to_owned())
                .to_string_lossy()
                .to_string();
            let path = INSTALL_DIRS.plugin_dir.join(&name_str);
            let index = model
                .iter::<gio::File>()
                .position(|x| x.unwrap().peek_path().is_some_and(|cur| &cur == &path));
            if let Some(index) = index {
                select.set_current_index(index as u32);
            }
        }

        fn save_value_one(
            model: &gio::ListStore,
            sect: &mut ConfigSectionMut<'_>,
            key: &CStr,
            select: &PluginSelect,
        ) {
            let index = match select.current_index() {
                gtk::INVALID_LIST_POSITION => 0,
                value => value,
            };

            let gio_path = model.item(index).and_downcast::<gio::File>().unwrap();
            let name_cstr = gio_path
                .basename()
                .map(|path| CString::new(path.to_string_lossy().as_bytes()))
                .unwrap()
                .unwrap();

            sect.set(key, &name_cstr).unwrap();
        }
    }

    impl SettingsPageImpl for PluginsPage {
        async fn load_page(&self, state: &mut CoreReadyState) {
            self.check_plugins().await;
            // TODO proper error handling
            let mut sect = state.cfg_open_mut(CFG_SECTION_KEY).unwrap();
            // load values
            Self::load_value_one(
                &self.graphics_plugins,
                &mut sect,
                CFG_GRAPHICS,
                &self.dd_graphics_plugins,
            );
            Self::load_value_one(
                &self.audio_plugins,
                &mut sect,
                CFG_AUDIO,
                &self.dd_audio_plugins,
            );
            Self::load_value_one(
                &self.input_plugins,
                &mut sect,
                CFG_INPUT,
                &self.dd_input_plugins,
            );
            Self::load_value_one(&self.rsp_plugins, &mut sect, CFG_RSP, &self.dd_rsp_plugins);
        }

        async fn save_page(&self, state: &mut CoreReadyState) {
            // TODO proper error handling
            let mut sect = state.cfg_open_mut(CFG_SECTION_KEY).unwrap();

            Self::save_value_one(
                &self.graphics_plugins,
                &mut sect,
                CFG_GRAPHICS,
                &self.dd_graphics_plugins,
            );
            Self::save_value_one(
                &self.audio_plugins,
                &mut sect,
                CFG_AUDIO,
                &self.dd_audio_plugins,
            );
            Self::save_value_one(
                &self.input_plugins,
                &mut sect,
                CFG_INPUT,
                &self.dd_input_plugins,
            );
            Self::save_value_one(&self.rsp_plugins, &mut sect, CFG_RSP, &self.dd_rsp_plugins);
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

/// Set the default config for this page.
pub(super) fn init_config(core: &mut Core) {
    let mut sect = core.cfg_open_mut(c"M64PRS-Plugins").unwrap();
    sect.set_default(
        c"Graphics",
        CString::new(add_lib_ext("mupen64plus-video-rice")).unwrap(),
        c"The graphics plugin to use with m64prs",
    )
    .unwrap();
    sect.set_default(
        c"Audio",
        CString::new(add_lib_ext("mupen64plus-audio-sdl")).unwrap(),
        c"The audio plugin to use with m64prs",
    )
    .unwrap();
    sect.set_default(
        c"Input",
        CString::new(add_lib_ext("mupen64plus-input-tasinput")).unwrap(),
        c"The input plugin to use with m64prs",
    )
    .unwrap();
    sect.set_default(
        c"RSP",
        CString::new(add_lib_ext("mupen64plus-rsp-hle")).unwrap(),
        c"The RSP plugin to use with m64prs",
    )
    .unwrap();
}
