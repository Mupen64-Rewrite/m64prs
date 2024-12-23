use gtk::prelude::*;

mod inner {
    use std::{
        cell::{Cell, RefCell},
        error::Error,
        pin::pin,
    };

    use futures::channel::oneshot;
    use glib::{subclass::InitializingObject, translate::IntoGlib};
    use gtk::{prelude::*, subclass::prelude::*, TemplateChild};
    use m64prs_vcr::movie::M64Header;

    use crate::{
        controls::SizedTextBuffer,
        ui::{movie_dialog::enums::MovieStartType, AppDialogError},
    };

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/ui/movie_dialog/window.blp")]
    #[properties(wrapper_type = super::MovieDialogWindow)]
    pub struct MovieDialogWindow {
        #[template_child]
        author_field: TemplateChild<gtk::TextView>,
        #[template_child]
        description_field: TemplateChild<gtk::TextView>,
        #[template_child]
        reset_btn: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        savestate_btn: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        eeprom_btn: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        file_dialog: TemplateChild<gtk::FileDialog>,
        #[template_child]
        error_dialog: TemplateChild<gtk::AlertDialog>,

        #[property(get, construct_only, default_value = false)]
        load: Cell<bool>,

        /// response values
        #[property(get, set, nullable)]
        cur_file: RefCell<Option<gio::File>>,
        #[property(get, set, builder(MovieStartType::Reset))]
        start_type: Cell<MovieStartType>,

        close_ok: Cell<bool>,
    }

    #[m64prs_gtk_utils::forward_wrapper(super::MovieDialogWindow, vis = pub(in super::super))]
    impl MovieDialogWindow {
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

        pub(super) fn author(&self) -> String {
            let buffer = self.author_field.buffer();
            buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), true)
                .to_string()
        }

        pub(super) fn description(&self) -> String {
            let buffer = self.description_field.buffer();
            buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), true)
                .to_string()
        }
    }

    #[gtk::template_callbacks]
    impl MovieDialogWindow {
        #[template_callback]
        fn sel_title(&self, load: bool) -> String {
            match load {
                true => "Load Movie...",
                false => "New Movie...",
            }
            .to_owned()
        }

        // This isn't directly supported by UI files, so it
        // needs to be defined here
        #[template_callback]
        fn not(&self, input: bool) -> bool {
            !input
        }

        #[template_callback]
        fn cond_string(&self, cond: bool, if_true: &str, if_false: &str) -> String {
            match cond {
                true => if_true,
                false => if_false,
            }
            .to_string()
        }

        #[template_callback]
        fn gio_file_path(&self, input: Option<gio::File>) -> String {
            input.as_ref().and_then(|file| file.path()).map_or_else(
                || "<unknown path>".to_string(),
                |path| path.to_string_lossy().to_string(),
            )
        }

        #[template_callback]
        fn start_type_eq(&self, start_type: MovieStartType, value: i32) -> bool {
            start_type.into_glib() == value
        }

        #[template_callback]
        async fn prompt_file(&self, _: &gtk::Button) {
            if let Err(err) = self.prompt_file_impl().await {
                self.error_dialog.set_message("Invalid file!");
                self.error_dialog.set_detail(&err.to_string());
                let _ = self.error_dialog.choose_future(Some(&*self.obj())).await;
            }
        }

        async fn prompt_file_impl(&self) -> Result<(), Box<dyn Error>> {
            let file = {
                let future = match self.load.get() {
                    true => self.file_dialog.open_future(Some(&*self.obj())),
                    false => self.file_dialog.save_future(Some(&*self.obj())),
                };
                match future.await {
                    Ok(path) => path,
                    Err(err) => match err.kind::<gtk::DialogError>().unwrap() {
                        gtk::DialogError::Dismissed => return Ok(()),
                        _ => return Err(err.into()),
                    },
                }
            };
            if file.path().is_none() {
                return Err(
                    AppDialogError("File has no path. Is your app sandboxed?".to_string()).into(),
                );
            }
            if self.load.get() {
                let header = {
                    let mut file_reader = file
                        .read_future(glib::Priority::DEFAULT)
                        .await?
                        .into_async_buf_read(1024);

                    M64Header::read_async(pin!(&mut file_reader)).await?
                };
                self.author_field.buffer().set_text(header.author.read());
                self.description_field
                    .buffer()
                    .set_text(header.description.read());
                if let Some(start_type) = MovieStartType::try_from(header.start_flags).ok() {
                    self.obj().set_start_type(start_type);
                }
            }
            self.obj().set_cur_file(Some(file));
            Ok(())
        }

        #[template_callback]
        fn ui_set_start_type(&self, button: &gtk::Button) {
            let start_type = if button == &*self.reset_btn {
                MovieStartType::Reset
            } else if button == &*self.savestate_btn {
                MovieStartType::Snapshot
            } else if button == &*self.eeprom_btn {
                MovieStartType::Eeprom
            } else {
                unreachable!()
            };
            self.obj().set_start_type(start_type);
        }

        #[template_callback]
        async fn ok_clicked(&self, _: &gtk::Button) {
            if self.obj().cur_file().is_none() {
                self.error_dialog.set_message("Missing info!");
                self.error_dialog.set_detail("Please select a file.");
                self.error_dialog.choose_future(Some(&*self.obj())).await;
                return;
            }

            self.close_ok.set(true);
            self.obj().set_visible(false);
        }
        #[template_callback]
        fn cancel_clicked(&self, _: &gtk::Button) {
            self.close_ok.set(false);
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
    impl WindowImpl for MovieDialogWindow {
        fn close_request(&self) -> glib::Propagation {
            self.close_ok.set(false);
            glib::Propagation::Proceed
        }
    }
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
    pub(super) fn with_load(load: bool) -> Self {
        let mut props: [(&str, glib::Value); 1] = [("load", load.to_value())];
        unsafe { glib::Object::with_mut_values(Self::static_type(), &mut props).unsafe_cast() }
    }
}
