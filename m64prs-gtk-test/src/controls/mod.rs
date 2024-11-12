mod native;

mod inner {
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk::subclass::widget::WidgetImpl;

    use super::native::PlatformSubsurface;


    pub struct ChildWindowContainer {
        subsurfaces: Vec<Box<dyn PlatformSubsurface>>,
    }

    impl Default for ChildWindowContainer {
        fn default() -> Self {
            Self {
                subsurfaces: vec![]
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChildWindowContainer {
        const NAME: &'static str = "M64PRSChildWindowContainer";
        type Type = super::ChildWindowContainer;
        type ParentType = gtk::Widget;

        
    }

    impl ObjectImpl for ChildWindowContainer {}

    impl WidgetImpl for ChildWindowContainer {}

}

glib::wrapper! {
    pub struct ChildWindowContainer(ObjectSubclass<inner::ChildWindowContainer>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}