use std::{future::Future, pin::Pin};

use futures::FutureExt;
use gtk::{prelude::*, subclass::prelude::*};

use crate::ui::core::CoreReadyState;

pub mod ffi {
    use futures::future::LocalBoxFuture;
    use gtk::subclass::prelude::*;

    use crate::ui::core::CoreReadyState;
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SettingsPageInterface {
        parent: glib::gobject_ffi::GTypeInterface,

        pub(super) load_page:
            for<'a> fn(&'a super::SettingsPage, &'a mut CoreReadyState) -> LocalBoxFuture<'a, ()>,
        pub(super) save_page:
            for<'a> fn(&'a super::SettingsPage, &'a mut CoreReadyState) -> LocalBoxFuture<'a, ()>,
    }

    unsafe impl InterfaceStruct for SettingsPageInterface {
        type Type = super::iface::SettingsPage;
    }
}

pub mod iface {
    use gtk::subclass::prelude::*;

    pub enum SettingsPage {}

    #[glib::object_interface]
    impl ObjectInterface for SettingsPage {
        const NAME: &'static str = "M64PRS_SettingsPage";
        type Prerequisites = (gtk::Widget,);
        type Interface = super::ffi::SettingsPageInterface;

        fn interface_init(class: &mut Self::Interface) {
            class.load_page =
                |_obj, _state| panic!("SettingsPage::load_from_core not implemented!");
            class.save_page = |_obj, _state| panic!("SettingsPage::save_to_core not implemented!");
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
    fn load_from_core<'a>(
        &'a self,
        core_state: &'a mut CoreReadyState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        let this = self.upcast_ref::<SettingsPage>();
        let iface = this.interface::<SettingsPage>().unwrap();
        (iface.as_ref().load_page)(this, core_state)
    }

    fn save_to_core<'a>(
        &'a self,
        core_state: &'a mut CoreReadyState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        let this = self.upcast_ref::<SettingsPage>();
        let iface = this.interface::<SettingsPage>().unwrap();
        (iface.as_ref().save_page)(this, core_state)
    }
}
impl<T: IsA<SettingsPage>> SettingsPageExt for T {}

pub trait SettingsPageImpl: WidgetImpl + ObjectSubclass<Type: IsA<SettingsPage>> {
    fn load_page(&self, state: &mut CoreReadyState) -> impl Future<Output = ()> {
        self.parent_load_page(state)
    }
    fn save_page(&self, state: &mut CoreReadyState) -> impl Future<Output = ()> {
        self.parent_save_page(state)
    }
}

pub trait SettingsPageImplExt: SettingsPageImpl {
    async fn parent_load_page(&self, state: &mut CoreReadyState) {
        let data = Self::type_data();
        let parent_iface = unsafe {
            &*(data.as_ref().parent_interface::<SettingsPage>()
                as *const ffi::SettingsPageInterface)
        };
        (parent_iface.load_page)(unsafe { self.obj().unsafe_cast_ref() }, state).await
    }
    async fn parent_save_page(&self, state: &mut CoreReadyState) {
        let data = Self::type_data();
        let parent_iface = unsafe {
            &*(data.as_ref().parent_interface::<SettingsPage>()
                as *const ffi::SettingsPageInterface)
        };
        (parent_iface.save_page)(unsafe { self.obj().unsafe_cast_ref() }, state).await
    }
}
impl<T: SettingsPageImpl> SettingsPageImplExt for T {}

unsafe impl<T: SettingsPageImpl> IsImplementable<T> for SettingsPage {
    fn interface_init(iface: &mut glib::Interface<Self>) {
        let class = iface.as_mut();

        class.load_page = |obj, state| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            SettingsPageImpl::load_page(this, state).boxed_local()
        };
        class.save_page = |obj, state| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            SettingsPageImpl::save_page(this, state).boxed_local()
        };
    }
}
