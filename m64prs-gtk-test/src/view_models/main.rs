use glib::subclass::types::FromObject;
use gtk::{prelude::*, subclass::prelude::*};

mod inner {

    use std::{
        cell::{Cell, RefCell},
        env,
        sync::{Arc, RwLock},
    };

    use super::*;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::MainViewModel)]
    pub struct MainViewModel {
        #[property(type = String, get, set)]
        title: RefCell<String>,
        #[property(type = bool, get, set)]
        resizable: Cell<bool>,

        core: RefCell<Option<Arc<RwLock<m64prs_core::Core>>>>,
    }

    impl Default for MainViewModel {
        fn default() -> Self {
            Self {
                title: String::from("m64prs").into(),
                resizable: false.into(),
                core: None.into(),
            }
        }
    }

    impl MainViewModel {
        pub(super) fn init_core(&self) {
            let self_path = env::current_exe().unwrap();
            let mupen_path = self_path
                .parent()
                .map(|file| {
                    #[cfg(target_os = "linux")]
                    const DLL_NAME: &str = "libmupen64plus.so";
                    #[cfg(target_os = "windows")]
                    const DLL_NAME: &str = "mupen64plus.dll";

                    file.join(DLL_NAME)
                })
                .unwrap();

            let core = m64prs_core::Core::init(mupen_path).expect("Mupen init failed!");

            // this is incredibly cursed
            let mut core_ref = self.core.borrow_mut();
            *core_ref = Some(Arc::new(RwLock::new(core)));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainViewModel {
        const NAME: &'static str = "M64prsMainViewModel";
        type Type = super::MainViewModel;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainViewModel {}
}

glib::wrapper! {
    pub struct MainViewModel(ObjectSubclass<inner::MainViewModel>);
}

impl MainViewModel {
    pub fn new() -> Self {
        let r = glib::Object::new::<MainViewModel>();
        inner::MainViewModel::from_object(&r).init_core();

        r
    }
}
