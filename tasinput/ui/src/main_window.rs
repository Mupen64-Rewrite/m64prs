use gtk::prelude::*;

macro_rules! refcell_bflags_get {
    ($this:ident, $button:ident) => {
        |this: &MainWindow| this.button_flags.borrow().contains(GButtonFlags::$button)
    };
}
macro_rules! refcell_bflags_set {
    ($this:ident, $button:ident) => {
        |this: &MainWindow, value| {
            this.button_flags
                .borrow_mut()
                .set(GButtonFlags::$button, value)
        }
    };
}

mod inner {
    use std::cell::{Cell, RefCell};

    use gtk::{prelude::*, subclass::prelude::*};
    use m64prs_sys::Buttons;

    use crate::{enums::GButtonFlags, joystick::Joystick};

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[template(file = "main_window.ui")]
    #[properties(wrapper_type = super::MainWindow)]
    pub struct MainWindow {
        #[property(
            name = "dr-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, D_RIGHT), 
            set = refcell_bflags_set!(this, D_RIGHT)
        )]
        #[property(
            name = "dl-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, D_LEFT), 
            set = refcell_bflags_set!(this, D_LEFT)
        )]
        #[property(
            name = "du-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, D_UP), 
            set = refcell_bflags_set!(this, D_UP)
        )]
        #[property(
            name = "dd-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, D_DOWN), 
            set = refcell_bflags_set!(this, D_DOWN)
        )]
        #[property(
            name = "start-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, START), 
            set = refcell_bflags_set!(this, START)
        )]
        #[property(
            name = "z-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, Z), 
            set = refcell_bflags_set!(this, Z)
        )]
        #[property(
            name = "b-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, B), 
            set = refcell_bflags_set!(this, B)
        )]
        #[property(
            name = "a-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, A), 
            set = refcell_bflags_set!(this, A)
        )]
        #[property(
            name = "cr-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, C_RIGHT), 
            set = refcell_bflags_set!(this, C_RIGHT)
        )]
        #[property(
            name = "cl-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, C_LEFT), 
            set = refcell_bflags_set!(this, C_LEFT)
        )]
        #[property(
            name = "cu-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, C_UP), 
            set = refcell_bflags_set!(this, C_UP)
        )]
        #[property(
            name = "cd-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, C_DOWN), 
            set = refcell_bflags_set!(this, C_DOWN)
        )]
        #[property(
            name = "r-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, R), 
            set = refcell_bflags_set!(this, R)
        )]
        #[property(
            name = "l-pressed", 
            type = bool,
            get = refcell_bflags_get!(this, L), 
            set = refcell_bflags_set!(this, L)
        )]
        button_flags: RefCell<GButtonFlags>,
        #[property(get, set)]
        joy_x: Cell<i8>,
        #[property(get, set)]
        joy_y: Cell<i8>,
    }

    #[m64prs_gtk_utils::forward_wrapper(super::MainWindow, vis = pub(crate))]
    impl MainWindow {
        pub(super) fn poll_input(&self) -> Buttons {
            Buttons {
                button_bits: self.button_flags.borrow().clone().into(),
                x_axis: self.joy_x.get(),
                y_axis: self.joy_y.get(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "TasDiMainWindow";
        type Type = super::MainWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(class: &mut Self::Class) {
            Joystick::ensure_type();

            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainWindow {
        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {
        fn close_request(&self) -> glib::Propagation {
            glib::Propagation::Stop
        }
    }
    impl ApplicationWindowImpl for MainWindow {}
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

    pub fn setup_and_show(app: &impl IsA<gtk::Application>) {
        Self::new(app).present();
    }
}
