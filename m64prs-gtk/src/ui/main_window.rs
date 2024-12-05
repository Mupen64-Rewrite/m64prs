use std::fmt::Display;

use glib::{translate::IntoGlib, EnumClass};
use gtk::prelude::*;

mod inner {
    use std::cell::Cell;

    use glib::subclass::{
        object::ObjectImpl,
        types::{ObjectSubclass, ObjectSubclassExt},
        InitializingObject,
    };
    use gtk::{prelude::*, subclass::prelude::*, TemplateChild};

    use crate::controls;

    use super::MainViewState;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[template(file = "src/ui/main_window.blp")]
    #[properties(wrapper_type = super::MainWindow)]
    pub struct MainWindow {
        #[template_child]
        rom_browser: TemplateChild<gtk::Widget>,
        #[template_child]
        compositor: TemplateChild<controls::CompositorView>,
        #[property(get, set, builder(MainViewState::RomBrowser))]
        #[property(
            get = |this: &MainWindow| this.current_view.get().to_string(), 
            type = String, 
            name = "current-view-str"
        )]
        current_view: Cell<MainViewState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "M64PRS_MainWindow";
        type Type = super::MainWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(class: &mut Self::Class) {
            
            class.bind_template();
        }

        fn instance_init(this: &InitializingObject<Self>) {
            this.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_default_size(640, 480);
            self.obj().set_title(Some("m64prs"));
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "M64PRS_MainViewState")]
pub enum MainViewState {
    #[enum_value(name = "rom-browser")]
    RomBrowser,
    #[enum_value(name = "game-view")]
    GameView,
}
impl Default for MainViewState {
    fn default() -> Self {
        Self::RomBrowser
    }
}
impl Display for MainViewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clazz = EnumClass::with_type(MainViewState::static_type()).unwrap();
        f.write_str(clazz.value(self.into_glib()).unwrap().name())
    }
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<inner::MainWindow>)
        @extends
            gtk::ApplicationWindow,
            gtk::Window,
            gtk::Widget,
        @implements
            gio::ActionGroup,
            gio::ActionMap,
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget,
            gtk::Native,
            gtk::Root,
            gtk::ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &impl IsA<gtk::Application>) -> Self {
        unsafe {
            glib::Object::with_mut_values(Self::static_type(), &mut [("application", app.into())])
                .unsafe_cast()
        }
    }
}
