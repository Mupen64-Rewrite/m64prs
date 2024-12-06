use std::error::Error;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;

use super::core::CoreState;

mod menu;
mod enums;

mod inner {
    use std::{cell::{Cell, RefCell}, error::Error};

    use glib::{subclass::{
        object::ObjectImpl,
        types::{ObjectSubclass, ObjectSubclassExt},
        InitializingObject,
    }, SendWeakRef};
    use gtk::{prelude::*, subclass::prelude::*, TemplateChild};
    use m64prs_sys::EmuState;

    use crate::{controls, ui::core::{CoreReadyState, CoreState}};

    use super::{enums::{MainEmuState, MainViewState}, menu::AppActions};

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[template(file = "src/ui/main_window/mod.blp")]
    #[properties(wrapper_type = super::MainWindow)]
    pub struct MainWindow {
        // template children
        #[template_child]
        rom_browser: TemplateChild<gtk::Widget>,
        #[template_child]
        compositor: TemplateChild<controls::CompositorView>,
        #[template_child]
        open_rom_dialog: TemplateChild<gtk::FileDialog>,
        #[template_child]
        error_dialog: TemplateChild<gtk::AlertDialog>,

        // properties
        #[property(get, set, builder(MainViewState::RomBrowser))]
        #[property(
            get = |this: &MainWindow| this.current_view.get().to_string(), 
            type = String, 
            name = "current-page"
        )]
        current_view: Cell<MainViewState>,
        #[property(get, builder(MainEmuState::Uninit))]
        emu_state: Cell<MainEmuState>,

        // private variables
        actions: AppActions,
        core: RefCell<CoreState>,
    }

    #[m64prs_gtk_macros::forward_wrapper(super::MainWindow, vis = pub(in crate::ui))]
    impl MainWindow {
        pub(super) fn set_emu_state(&self, emu_state: EmuState) {
            self.emu_state.set(emu_state.into());
            self.obj().notify_emu_state();
        }

        pub(super) fn exec_with_core<R, F: FnOnce(&mut CoreState) -> R>(&self, f: F) -> R {
            let mut core = self.core.borrow_mut();
            f(&mut *core)
        }

        pub(super) async fn show_open_rom_dialog(&self) -> Result<gio::File, glib::Error> {
            self.open_rom_dialog.open_future(Some(&self.obj().clone())).await
        }

        pub(super) async fn show_error_dialog(&self, header: &str, error: &dyn Error) {
            self.error_dialog.set_message(header);
            self.error_dialog.set_detail(&error.to_string());
            let _ = self.error_dialog.choose_future(Some(&self.obj().clone())).await;
        }
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
            self.actions.init(&self.obj());

            {
                let this = self.obj().clone();
                glib::spawn_future_local(async move {
                    let self_weak_ref: SendWeakRef<_> = this.downgrade().into();
                    let ready_state = gio::spawn_blocking(move || {
                        CoreReadyState::new(self_weak_ref)
                    }).await.expect("failed to init core");
                    this.set_emu_state(EmuState::Stopped);
                    this.imp().core.replace(ready_state.into());
                });
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }

        fn notify(&self, pspec: &glib::ParamSpec) {
            self.parent_notify(pspec);

            match pspec.name() {
                "application" => {
                    if let Some(app) = self.obj().application() {
                        self.actions.register_to(&app);
                    }
                }
                "emu-state" => {
                    let value = self.obj().emu_state();
                    if value == MainEmuState::Stopped {
                        let mut core = self.core.borrow_mut();

                        match core.take() {
                            CoreState::Running(core_running_state) => {
                                *core = CoreState::Ready(core_running_state.stop_rom().0);
                            },
                            other_state => *core = other_state, 
                        }
                    }
                }
                _ => (),
            }
            
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

    pub fn setup_and_show(app: &impl IsA<gtk::Application>) {
        let menu = menu::load_menu();
        app.set_menubar(Some(&menu));

        let window: Self = unsafe {
            glib::Object::with_mut_values(Self::static_type(), &mut [("application", app.into())])
                .unsafe_cast()
        };
        window.set_show_menubar(true);

        window.present();
    }

    // pub(in crate::ui) fn set_emu_state(&self, emu_state: m64prs_sys::EmuState) {
    //     println!("emu state: {:?}", emu_state);
    //     self.imp().set_emu_state(emu_state)
    // }

    // pub(in crate::ui) fn exec_with_core<R, F: FnOnce(&mut CoreState) -> R>(&self, f: F) -> R {
    //     // self.imp().exec_with_core(f)
    //     glib::subclass::types::ObjectSubclassIsExt::imp(self)
    //         .exec_with_core(f)
    // }

    // pub(in crate::ui) async fn show_open_rom_dialog(&self) -> Result<gio::File, glib::Error> {
    //     self.imp().show_open_rom_dialog().await
    // }

    // pub(in crate::ui) async fn show_error_dialog(&self, header: &str, error: &dyn Error) {
    //     self.imp().show_error_dialog(header, error).await
    // }
}
