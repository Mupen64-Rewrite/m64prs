use std::{cell::RefMut, error::Error};

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;

use super::core::CoreState;

pub mod enums;
mod menu;

mod inner {
    use std::{
        cell::{Cell, RefCell, RefMut},
        error::Error,
    };

    use glib::{
        subclass::{
            object::ObjectImpl,
            types::{ObjectSubclass, ObjectSubclassExt},
            InitializingObject,
        },
        SendWeakRef,
    };
    use gtk::{prelude::*, subclass::prelude::*, TemplateChild};
    use m64prs_sys::EmuState;

    use crate::{
        controls::{
            self,
            compositor_view::native::{NativeView, NativeViewAttributes, NativeViewKey},
        },
        ui::{core::{CoreReadyState, CoreState}, movie_dialog::MovieDialog},
    };

    use super::{
        enums::{MainEmuState, MainViewState},
        menu::AppActions,
    };

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
        #[template_child]
        save_state_dialog: TemplateChild<gtk::FileDialog>,
        #[template_child]
        load_state_dialog: TemplateChild<gtk::FileDialog>,
        #[template_child]
        movie_dialog: TemplateChild<MovieDialog>,

        // properties
        #[property(get, construct_only, builder(MainViewState::RomBrowser))]
        #[property(
            get = |this: &MainWindow| this.current_view.get().to_string(),
            type = String,
            name = "current-page"
        )]
        current_view: Cell<MainViewState>,
        #[property(get, construct_only, builder(MainEmuState::Uninit))]
        emu_state: Cell<MainEmuState>,
        #[property(get, construct_only, default = false)]
        saving_state: Cell<bool>,
        #[property(get, construct_only, default = 1)]
        save_slot: Cell<u8>,
        #[property(get, construct_only, default = false)]
        vcr_active: Cell<bool>,
        #[property(get, construct_only, default = false)]
        vcr_read_only: Cell<bool>,

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

        pub(super) fn set_saving_state(&self, saving_state: bool) {
            self.saving_state.set(saving_state);
            self.obj().notify_saving_state();
        }

        pub(super) fn set_save_slot(&self, save_slot: u8) {
            self.save_slot.set(save_slot);
            self.obj().notify_save_slot();
        }

        pub(super) fn set_vcr_read_only(&self, vcr_read_only: bool) {
            self.vcr_read_only.set(vcr_read_only);
            self.obj().notify_vcr_read_only();
        }

        pub(super) fn set_current_view(&self, main_view: MainViewState) {
            self.current_view.set(main_view);
            {
                let obj = self.obj();
                obj.notify_current_view();
                obj.notify_current_page();
            }
        }

        pub(super) fn borrow_core<'a>(&'a self) -> RefMut<'a, CoreState> {
            self.core.borrow_mut()
        }

        pub(super) fn comp_new_view(&self, attrs: NativeViewAttributes) -> Box<dyn NativeView> {
            self.compositor.new_view(attrs)
        }

        pub(super) fn comp_del_view(&self, view: NativeViewKey) {
            self.compositor.del_view(view);
        }

        pub(super) async fn show_open_rom_dialog(&self) -> Result<gio::File, glib::Error> {
            self.open_rom_dialog.open_future(Some(&*self.obj())).await
        }

        pub(super) async fn show_error_dialog(&self, header: &str, error: &dyn Error) {
            self.error_dialog.set_message(header);
            self.error_dialog.set_detail(&error.to_string());
            let _ = self.error_dialog.choose_future(Some(&*self.obj())).await;
        }

        pub(super) async fn show_save_state_dialog(&self) -> Result<gio::File, glib::Error> {
            self.save_state_dialog.save_future(Some(&*self.obj())).await
        }

        pub(super) async fn show_load_state_dialog(&self) -> Result<gio::File, glib::Error> {
            self.load_state_dialog.open_future(Some(&*self.obj())).await
        }

        pub(super) async fn show_movie_dialog(&self) {
            self.movie_dialog.prompt(Some(&*self.obj())).await;
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
                let kc = gtk::EventControllerKey::new();
                kc.connect_key_pressed({
                    let this = self.obj().downgrade();
                    move |_, _key, code, r#mod| {
                        let this = match this.upgrade() {
                            Some(this) => this,
                            None => return glib::Propagation::Proceed,
                        };

                        eprintln!("key: {}, {}", _key.name().unwrap().as_str(), code);

                        if let Some(running) = this.borrow_core().borrow_running() {
                            running.forward_key_down(code, r#mod);
                        }
                        glib::Propagation::Stop
                    }
                });
                kc.connect_key_released({
                    let this = self.obj().downgrade();
                    move |_, _key, code, r#mod| {
                        let this = match this.upgrade() {
                            Some(this) => this,
                            None => return,
                        };

                        if let Some(running) = this.borrow_core().borrow_running() {
                            running.forward_key_up(code, r#mod);
                        }
                        // this exists to please the borrow checker. Don't ask me why.
                        ()
                    }
                });
                self.compositor.add_controller(kc);
            }

            {
                let this = self.obj().clone();
                glib::spawn_future_local(async move {
                    let self_weak_ref: SendWeakRef<_> = this.downgrade().into();
                    let ready_state =
                        gio::spawn_blocking(move || CoreReadyState::new(self_weak_ref))
                            .await
                            .expect("failed to init core");
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
                "current-view" => {
                    let obj = self.obj();
                    match obj.current_view() {
                        MainViewState::RomBrowser => {
                            self.grab_focus();
                        }
                        MainViewState::GameView => {
                            self.compositor.grab_focus();
                        }
                    }
                }
                "emu-state" => {
                    let value = self.obj().emu_state();
                    if value == MainEmuState::Stopped {
                        let self_ref = self.obj().clone();
                        glib::spawn_future_local(async move {
                            let mut core = self_ref.imp().core.borrow_mut();
                            match core.take() {
                                CoreState::Running(core_running_state) => {
                                    *core = CoreState::Ready(core_running_state.stop_rom().await.0);
                                }
                                other_state => *core = other_state,
                            }
                        });
                    }
                }
                "focus-widget" => {
                    // This may seem incredibly cursed but it works
                    // (if the focus is not the compositor, then change it to the compositor)
                    if self.current_view.get() == MainViewState::GameView
                        && !gtk::prelude::GtkWindowExt::focus(&*self.obj())
                            .is_some_and(|w| &w == &*self.compositor)
                    {
                        self.compositor.grab_focus();
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
}
