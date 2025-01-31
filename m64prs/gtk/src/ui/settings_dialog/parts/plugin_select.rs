mod inner {
    use std::{
        cell::{Cell, RefCell},
        sync::LazyLock,
    };

    use glib::subclass::Signal;
    use gtk::{prelude::*, subclass::prelude::*};

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "plugin_select.ui")]
    #[properties(wrapper_type = super::PluginSelect)]
    pub struct PluginSelect {
        #[property(get, set = |this: &PluginSelect, value: &gio::ListModel| {
            assert!(value.item_type().is_a(gio::File::static_type()));
            this.plugins.replace(value.clone());
        }, type = gio::ListModel)]
        plugins: RefCell<gio::ListModel>,
        #[property(get, set)]
        settings_available: Cell<bool>,
        #[property(get, set)]
        current_index: Cell<u32>,
    }

    #[gtk::template_callbacks]
    impl PluginSelect {
        #[template_callback]
        fn path_to_string(file: &gio::File) -> String {
            file.peek_path()
                .as_ref()
                .and_then(|path| path.file_name())
                .map_or("".to_string(), |name| name.to_string_lossy().to_string())
        }

        #[template_callback]
        fn open_settings_btn(&self, btn: &gtk::Button) {
            self.obj()
                .emit(<Self as ObjectImpl>::signals()[0].signal_id(), &[])
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginSelect {
        const NAME: &'static str = "M64PRS_PluginSelect";

        type Type = super::PluginSelect;
        type ParentType = gtk::Frame;

        fn new() -> Self {
            let plugins = gio::ListStore::new::<gio::File>();
            // plugins.append(&gio::File::for_path(
            //     "/usr/lib/mupen64plus/mupen64plus-video-rice.so",
            // ));
            // plugins.append(&gio::File::for_path(
            //     "/usr/lib/mupen64plus/mupen64plus-video-glide64mk2.so",
            // ));
            Self {
                plugins: RefCell::new(plugins.upcast()),
                settings_available: Cell::new(false),
                current_index: Cell::new(gtk::INVALID_LIST_POSITION),
            }
        }

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
            class.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PluginSelect {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<[Signal; 1]> =
                LazyLock::new(|| [Signal::builder("open-settings").build()]);
            &*SIGNALS
        }
    }
    impl WidgetImpl for PluginSelect {}
    impl FrameImpl for PluginSelect {}
}

glib::wrapper! {
    pub struct PluginSelect(ObjectSubclass<inner::PluginSelect>)
        @extends
            gtk::Frame,
            gtk::Widget,
        @implements
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget;
}

impl PluginSelect {
    pub fn new() {}
}
