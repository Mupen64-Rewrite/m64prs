mod inner {
    use std::cell::{Cell, RefCell};

    use futures::channel::oneshot;
    use glib::translate::{FromGlib, IntoGlib};
    use gtk::{prelude::*, subclass::prelude::*};

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "window.ui")]
    #[properties(wrapper_type = super::AccelInputDialogWindow)]
    pub struct AccelInputDialogWindow {
        #[property(get, set)]
        key: Cell<u32>,
        #[property(get, set)]
        modifiers: Cell<gdk::ModifierType>,

        close_ok: Cell<bool>,
    }

    #[m64prs_gtk_utils::forward_wrapper(super::AccelInputDialogWindow, vis = pub(in super::super))]
    impl AccelInputDialogWindow {
        pub(super) async fn prompt(&self, transient_for: Option<&impl IsA<gtk::Window>>) -> bool {
            let (tx, rx) = oneshot::channel();

            self.obj().set_transient_for(transient_for);
            let handler_id = self.obj().connect_hide({
                let tx = RefCell::new(Some(tx));
                let this = self.obj().downgrade();
                move |_| {
                    let this = this.upgrade().unwrap();
                    if let Some(tx) = tx.take() {
                        let _ = tx.send(this.imp().close_ok.get());
                    }
                }
            });
            self.obj().present();
            let result = rx.await.unwrap();
            self.obj().disconnect(handler_id);

            result
        }
    }

    #[gtk::template_callbacks]
    impl AccelInputDialogWindow {
        #[template_callback]
        fn key_down(
            &self,
            keyval: gdk::Key,
            _: u32,
            mods: gdk::ModifierType,
            _: gtk::EventControllerKey,
        ) -> glib::Propagation {
            let mut real_mods = mods;
            match keyval {
                gdk::Key::Alt_L | gdk::Key::Alt_R => {
                    real_mods |= gdk::ModifierType::ALT_MASK;
                }
                gdk::Key::Control_L | gdk::Key::Control_R => {
                    real_mods |= gdk::ModifierType::CONTROL_MASK;
                }
                gdk::Key::Shift_L | gdk::Key::Shift_R => {
                    real_mods |= gdk::ModifierType::SHIFT_MASK;
                }
                gdk::Key::Meta_L | gdk::Key::Meta_R => {
                    real_mods |= gdk::ModifierType::META_MASK;
                }
                _ => (),
            }

            self.obj().set_modifiers(real_mods);

            match keyval {
                gdk::Key::Escape => {
                    self.close_ok.set(false);
                    self.obj().set_visible(false);
                    return glib::Propagation::Stop;
                }
                gdk::Key::Alt_L
                | gdk::Key::Alt_R
                | gdk::Key::Control_L
                | gdk::Key::Control_R
                | gdk::Key::Shift_L
                | gdk::Key::Shift_R
                | gdk::Key::Meta_L
                | gdk::Key::Meta_R => (),
                _ => {
                    self.obj().set_key(keyval.into_glib());
                    if gtk::accelerator_valid(keyval, mods) {
                        self.close_ok.set(true);
                        self.obj().set_visible(false);
                    }
                }
            }

            glib::Propagation::Proceed
        }

        #[template_callback]
        fn key_up(
            &self,
            keyval: gdk::Key,
            _: u32,
            mods: gdk::ModifierType,
            _: gtk::EventControllerKey,
        ) {
            let mut real_mods = mods;
            match keyval {
                gdk::Key::Alt_L | gdk::Key::Alt_R => {
                    real_mods &= !gdk::ModifierType::ALT_MASK;
                }
                gdk::Key::Control_L | gdk::Key::Control_R => {
                    real_mods &= !gdk::ModifierType::CONTROL_MASK;
                }
                gdk::Key::Shift_L | gdk::Key::Shift_R => {
                    real_mods &= !gdk::ModifierType::SHIFT_MASK;
                }
                gdk::Key::Meta_L | gdk::Key::Meta_R => {
                    real_mods &= !gdk::ModifierType::META_MASK;
                }
                _ => (),
            }
            self.obj().set_modifiers(real_mods);

            if !matches!(
                keyval,
                gdk::Key::Alt_L
                    | gdk::Key::Alt_R
                    | gdk::Key::Control_L
                    | gdk::Key::Control_R
                    | gdk::Key::Shift_L
                    | gdk::Key::Shift_R
                    | gdk::Key::Meta_L
                    | gdk::Key::Meta_R
            ) {
                self.obj().set_key(0);
            }
        }

        #[template_callback]
        fn cancel_clicked(&self, _: gtk::Button) {
            self.close_ok.set(false);
            self.obj().set_visible(false);
        }

        #[template_callback]
        fn unbind_clicked(&self, _: gtk::Button) {
            self.close_ok.set(true);
            self.obj().set_key(0);
            self.obj().set_modifiers(gdk::ModifierType::empty());
            self.obj().set_visible(false);
        }

        #[template_callback]
        fn get_label(&self, key: u32, mods: gdk::ModifierType) -> String {
            let label =
                gtk::accelerator_get_label(unsafe { gdk::Key::from_glib(key) }, mods).to_string();
            label
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccelInputDialogWindow {
        const NAME: &'static str = "M64PRS_AccelInputDialogWindow";
        type Type = super::AccelInputDialogWindow;
        type ParentType = gtk::Window;

        fn new() -> Self {
            Self {
                key: Cell::new(0),
                modifiers: Cell::new(gdk::ModifierType::empty()),
                close_ok: Cell::new(false),
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
    impl ObjectImpl for AccelInputDialogWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for AccelInputDialogWindow {}
    impl WindowImpl for AccelInputDialogWindow {}
}

glib::wrapper! {
    pub struct AccelInputDialogWindow(ObjectSubclass<inner::AccelInputDialogWindow>)
    @extends
        gtk::Window,
        gtk::Widget,
    @implements
        gtk::Accessible,
        gtk::Buildable,
        gtk::ConstraintTarget,
        gtk::Native,
        gtk::Root,
        gtk::ShortcutManager;
}

impl AccelInputDialogWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
