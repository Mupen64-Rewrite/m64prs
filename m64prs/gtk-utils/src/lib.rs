pub mod actions;
pub mod error;
pub mod menu;
pub mod t_option;

pub use m64prs_gtk_macros::forward_wrapper;

#[macro_export]
macro_rules! glib_enum_display {
    ($type:ty) => {
        impl ::std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let clazz = ::glib::EnumClass::with_type(
                    <$type as ::glib::prelude::StaticType>::static_type(),
                )
                .unwrap();
                f.write_str(
                    clazz
                        .value(::glib::translate::IntoGlib::into_glib(*self))
                        .unwrap()
                        .name(),
                )
            }
        }
    };
}
