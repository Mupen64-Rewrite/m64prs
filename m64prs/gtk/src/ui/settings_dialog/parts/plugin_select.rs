mod inner {
    use gtk::{prelude::*, subclass::prelude::*};


    #[derive(Default)]
    pub struct PluginSelect {

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginSelect {
        const NAME: &'static str = "M64PRS_PluginSelect";
    
        type Type = super::PluginSelect;
        type ParentType = gtk::Frame;
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