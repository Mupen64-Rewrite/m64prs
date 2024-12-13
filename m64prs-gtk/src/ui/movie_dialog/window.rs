use gtk::prelude::*;

use super::MovieDialog;

mod inner {
    use std::cell::{OnceCell, RefCell};

    use futures::channel::oneshot;
    use glib::subclass::InitializingObject;
    use gtk::{
        prelude::*,
        subclass::prelude::*,
        TemplateChild,
    };

    use crate::{controls::SizedTextBuffer, ui::movie_dialog::MovieDialog};

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/ui/movie_dialog/window.blp")]
    #[properties(wrapper_type = super::MovieDialogWindow)]
    pub struct MovieDialogWindow {
        #[template_child]
        author_field: TemplateChild<gtk::TextView>,
        #[template_child]
        description_field: TemplateChild<gtk::TextView>,

        #[property(get, construct_only, name = "props")]
        props: OnceCell<MovieDialog>,
    }

    #[m64prs_gtk_macros::forward_wrapper(super::MovieDialogWindow, vis = pub(in crate::ui))]
    impl MovieDialogWindow {
        pub(super) async fn prompt(&self, transient_for: Option<&impl IsA<gtk::Window>>) {
            let (tx, rx) = oneshot::channel();

            self.obj().set_transient_for(transient_for);
            let handler_id = self.obj().connect_hide({
                let tx = RefCell::new(Some(tx));
                move |_| {
                    if let Some(tx) = tx.take() {
                        let _ = tx.send(());
                    }
                }
            });
            self.obj().present();
            let _ = rx.await;
            self.obj().disconnect(handler_id);
        }
    }

    #[gtk::template_callbacks]
    impl MovieDialogWindow {
        #[template_callback]
        fn ok_pressed(&self) {}
        #[template_callback]
        fn cancel_pressed(&self) {
            self.obj().set_visible(false);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MovieDialogWindow {
        const NAME: &'static str = "M64PRS_MovieDialogWindow";
        type Type = super::MovieDialogWindow;
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            SizedTextBuffer::ensure_type();

            class.bind_template();
            class.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MovieDialogWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_property("hide-on-close", true);
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for MovieDialogWindow {}
    impl WindowImpl for MovieDialogWindow {}
}

glib::wrapper! {
    pub struct MovieDialogWindow(ObjectSubclass<inner::MovieDialogWindow>)
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

impl MovieDialogWindow {
    pub fn with_props(props: &MovieDialog) -> Self {
        unsafe {
            glib::Object::with_mut_values(Self::static_type(), &mut [("props", props.to_value())])
                .unsafe_cast()
        }
    }
}
