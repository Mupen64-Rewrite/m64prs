use gtk::{prelude::*, subclass::prelude::*};

mod inner {
    use glib::translate::FromGlib;

    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::AccelModel)]
    pub struct AccelModel {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        action: RefCell<String>,
        #[property(get, set)]
        #[property(name = "label", type = String, get = |this: &Self| unsafe {
            let key = this.key.get();
            if key == 0 {
                "[none]".to_string()
            }
            else {
                gtk::accelerator_get_label(gdk::Key::from_glib(this.key.get()), this.modifiers.get()).to_string()
            }
            
        })]
        key: Cell<u32>,
        #[property(get, set)]
        modifiers: Cell<gdk::ModifierType>,
    }

    impl Default for AccelModel {
        fn default() -> Self {
            Self {
                name: Default::default(),
                action: Default::default(),
                key: Default::default(),
                modifiers: Cell::new(gdk::ModifierType::NO_MODIFIER_MASK),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccelModel {
        const NAME: &'static str = "M64PRS_AccelModel";
        type Type = super::AccelModel;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AccelModel {
        fn notify(&self, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "key" | "modifiers" => {
                    self.obj().notify_label();
                }
                _ => (),
            }
        }
    }
}

glib::wrapper! {
    pub struct AccelModel(ObjectSubclass<inner::AccelModel>);
}

impl AccelModel {
    pub fn new(name: &str, action: &str) -> Self {
        // SAFETY: glib::Object::with_mut_value always
        // produces an object of type Self.
        unsafe {
            glib::Object::with_mut_values(
                Self::static_type(),
                &mut [("name", name.to_value()), ("action", action.to_value())],
            )
            .unsafe_cast()
        }
    }
}
