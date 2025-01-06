mod inner {
    use std::cell::RefCell;

    use gtk::{prelude::*, subclass::prelude::*};

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "plugin_select.ui")]
    #[properties(wrapper_type = super::PluginSelect)]
    pub struct PluginSelect {
        #[property(get)]
        plugins: gio::ListStore,
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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginSelect {
        const NAME: &'static str = "M64PRS_PluginSelect";

        type Type = super::PluginSelect;
        type ParentType = gtk::Frame;

        fn new() -> Self {
            Self {
                plugins: gio::ListStore::new::<gio::File>(),
            }
        }

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginSelect {}
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
