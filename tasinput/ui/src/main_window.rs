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
    use m64prs_sys::ButtonFlags;

    use crate::{enums::GButtonFlags, joystick::Joystick};

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[template(file = "src/main_window.blp")]
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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let ct_win_move = gtk::EventControllerLegacy::new();
            ct_win_move.set_propagation_phase(gtk::PropagationPhase::Bubble);
            ct_win_move.connect_event({
                let this = obj.downgrade();
                move |_, evt: &gdk::Event| {

                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return glib::Propagation::Proceed,
                    };
                    let toplevel = match this.surface().unwrap().downcast::<gdk::Toplevel>() {
                        Ok(toplevel) => toplevel,
                        Err(_) => return glib::Propagation::Proceed,
                    };

                    if let Some(pointer_evt) = evt.downcast_ref::<gdk::ButtonEvent>() {
                        // Get the button, only allow left click
                        let button = pointer_evt.button();
                        if button != gdk::BUTTON_PRIMARY {
                            return glib::Propagation::Proceed;
                        }
                        // gather other information
                        let device = match pointer_evt.device() {
                            Some(device) => device,
                            None => return glib::Propagation::Proceed,
                        };
                        let (x, y) = pointer_evt.position().unwrap();
                        if !this.pick(x, y, gtk::PickFlags::INSENSITIVE).is_some_and(|w| {
                            w.first_child().is_some()
                        }) {
                            return glib::Propagation::Proceed;
                        }
                        let timestamp = pointer_evt.time();

                        // request to move window
                        toplevel.begin_move(&device, button as i32, x, y, timestamp);

                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                }
            });

            obj.add_controller(ct_win_move);
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
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
    pub fn setup_and_show(app: &impl IsA<gio::Application>) {
        let window: Self = unsafe {
            glib::Object::with_mut_values(Self::static_type(), &mut [("application", app.into())])
                .unsafe_cast()
        };
        window.present();
    }
}
