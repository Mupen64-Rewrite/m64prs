mod inner {
    use std::{cell::{Cell, RefCell}, ffi::c_ulong, future::Future, rc::Rc, sync::{atomic::AtomicU64, Arc}};

    use futures::{channel::oneshot, FutureExt};
    use glib::{object::{IsA, ObjectExt}, subclass::{object::{ObjectImpl, ObjectImplExt}, types::{ObjectSubclass, ObjectSubclassExt}, InitializingObject}, SignalHandlerId};
    use gtk::{prelude::{GtkWindowExt, WidgetExt}, subclass::{
        widget::{
            CompositeTemplateClass, CompositeTemplateDisposeExt, CompositeTemplateInitializingExt,
            WidgetImpl,
        },
        window::WindowImpl,
    }};

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/ui/movie_dialog/mod.blp")]
    pub struct MovieDialog {}

    #[m64prs_gtk_macros::forward_wrapper(super::MovieDialog, vis = pub(in crate::ui))]
    impl MovieDialog {
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

    #[glib::object_subclass]
    impl ObjectSubclass for MovieDialog {
        const NAME: &'static str = "M64PRS_MovieDialog";
        type Type = super::MovieDialog;
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MovieDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_property("hide-on-close", true);
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for MovieDialog {}
    impl WindowImpl for MovieDialog {}
}

glib::wrapper! {
    pub struct MovieDialog(ObjectSubclass<inner::MovieDialog>)
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

impl MovieDialog {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }
}