mod pages;
mod parts;
mod settings_page;

pub(self) use settings_page::SettingsPage;

mod inner {
    use gtk::{prelude::*, subclass::prelude::*};

    use crate::ui::main_window::MainWindow;

    use super::{pages, settings_page::SettingsPageExt, SettingsPage};

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "mod.ui")]
    pub struct SettingsDialog {
        #[template_child]
        tabs_nb: TemplateChild<gtk::Notebook>,
    }

    impl SettingsDialog {
        fn main_window(&self) -> MainWindow {
            let parent = self.obj().transient_for().expect("SettingsDialog should have a parent window");
            parent.downcast().expect("parent window is not a MainWindow")
        }

        fn load_pages(&self) {
            let main_window = self.main_window();
            let mut core = main_window.borrow_core();
            let core_ready = core.borrow_ready().expect("Core should not be running");

            for page in &self.tabs_nb.pages() {
                let page: gtk::NotebookPage = page.unwrap().downcast().unwrap();
                
                if let Some(settings_page) = page.child().dynamic_cast_ref::<SettingsPage>() {
                    settings_page.load_from_core(core_ready);
                }
            }
        }
        fn save_pages(&self) {
            let main_window = self.main_window();
            let mut core = main_window.borrow_core();
            let core_ready = core.borrow_ready().expect("Core should not be running");

            for page in &self.tabs_nb.pages() {
                let page: gtk::NotebookPage = page.unwrap().downcast().unwrap();
                
                if let Some(settings_page) = page.child().dynamic_cast_ref::<SettingsPage>() {
                    settings_page.save_to_core(core_ready);
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsDialog {
        const NAME: &'static str = "M64PRS_SettingsDialog";
        type Type = super::SettingsDialog;
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            pages::init_pages();
            class.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsDialog {}
    impl WidgetImpl for SettingsDialog {
        fn map(&self) {
            self.parent_map();
            self.load_pages();
        }
    }
    impl WindowImpl for SettingsDialog {
        fn close_request(&self) -> glib::Propagation {
            glib::Propagation::Proceed
        }
    }
}

glib::wrapper! {
    pub struct SettingsDialog(ObjectSubclass<inner::SettingsDialog>)
        @extends
            gtk::Window,
            gtk::Widget,
        @implements
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget,
            gtk::Native,
            gtk::Root;
}

impl SettingsDialog {
    pub fn new() -> SettingsDialog {
        glib::Object::new::<SettingsDialog>()
    }
}
