use gtk::{prelude::*, subclass::prelude::*};

use crate::ui::core::CoreReadyState;

pub mod ffi {
    use gtk::{prelude::*, subclass::prelude::*};

    use crate::ui::core::CoreReadyState;
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SettingsPageInterface {
        parent: glib::gobject_ffi::GTypeInterface,

        pub(super) load_from_core: fn(&super::SettingsPage, &mut CoreReadyState),
        pub(super) save_to_core: fn(&super::SettingsPage, &mut CoreReadyState),
    }

    unsafe impl InterfaceStruct for SettingsPageInterface {
        type Type = super::iface::SettingsPage;
    }
}

pub mod iface {
    use gtk::{prelude::*, subclass::prelude::*};

    pub enum SettingsPage {}

    #[glib::object_interface]
    impl ObjectInterface for SettingsPage {
        const NAME: &'static str = "M64PRS_SettingsPage";
        type Prerequisites = (gtk::Widget,);
        type Interface = super::ffi::SettingsPageInterface;

        fn interface_init(class: &mut Self::Interface) {
            class.load_from_core = |_obj, _state| panic!("SettingsPage::load_from_core not implemented!");
            class.save_to_core = |_obj, _state| panic!("SettingsPage::save_to_core not implemented!");
        }
    }
}

glib::wrapper! {
    pub struct SettingsPage(ObjectInterface<iface::SettingsPage>)
        @requires
            gtk::Widget,
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget;
}

pub trait SettingsPageExt: IsA<SettingsPage> {
    fn load_from_core(&self, core_state: &mut CoreReadyState) {
        let this = self.upcast_ref::<SettingsPage>();
        let iface = this.interface::<SettingsPage>().unwrap();
        (iface.as_ref().load_from_core)(this, core_state);
    }

    fn save_to_core(&self, core_state: &mut CoreReadyState) {
        let this = self.upcast_ref::<SettingsPage>();
        let iface = this.interface::<SettingsPage>().unwrap();
        (iface.as_ref().save_to_core)(this, core_state);
    }
}
impl<T: IsA<SettingsPage>> SettingsPageExt for T {}

pub trait SettingsPageImpl: WidgetImpl + ObjectSubclass<Type: IsA<SettingsPage>> {
    fn load_from_core(&self, state: &mut CoreReadyState) {
        self.parent_load_from_core(state)
    }
    fn save_to_core(&self, state: &mut CoreReadyState) {
        self.parent_save_to_core(state)
    }
}

pub trait SettingsPageImplExt: SettingsPageImpl {
    fn parent_load_from_core(&self, state: &mut CoreReadyState) {
        let data = Self::type_data();
        let parent_iface = unsafe {
            &*(data.as_ref().parent_interface::<SettingsPage>()
                as *const ffi::SettingsPageInterface)
        };
        (parent_iface.load_from_core)(unsafe { self.obj().unsafe_cast_ref() }, state)
    }
    fn parent_save_to_core(&self, state: &mut CoreReadyState) {
        let data = Self::type_data();
        let parent_iface = unsafe {
            &*(data.as_ref().parent_interface::<SettingsPage>()
                as *const ffi::SettingsPageInterface)
        };
        (parent_iface.save_to_core)(unsafe { self.obj().unsafe_cast_ref() }, state)
    }
}
impl<T: SettingsPageImpl> SettingsPageImplExt for T {}

unsafe impl<T: SettingsPageImpl> IsImplementable<T> for SettingsPage {
    fn interface_init(iface: &mut glib::Interface<Self>) {
        let class = iface.as_mut();

        class.load_from_core = |obj, state| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            SettingsPageImpl::load_from_core(this, state);
        };
        class.save_to_core = |obj, state| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            SettingsPageImpl::save_to_core(this, state);
        };
    }
}