use glib::prelude::*;

macro_rules! glib_enum_display {
    ($type:ty) => {
        impl ::std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let clazz = ::glib::EnumClass::with_type(<$type>::static_type()).unwrap();
                f.write_str(clazz.value(::glib::translate::IntoGlib::into_glib(*self)).unwrap().name())
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "M64PRS_MainViewState")]
#[repr(u8)]
pub enum MainViewState {
    #[enum_value(name = "rom-browser")]
    RomBrowser = 0,
    #[enum_value(name = "game-view")]
    GameView = 1,
}
impl Default for MainViewState {
    fn default() -> Self {
        Self::RomBrowser
    }
}
glib_enum_display!(MainViewState);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "M64PRS_CoreEmuState")]
#[repr(u8)]
pub enum CoreEmuState {
    Uninit = 0,
    Stopped = 1,
    Running = 2,
    Paused = 3
}
impl Default for CoreEmuState {
    fn default() -> Self {
        CoreEmuState::Uninit
    }
}
impl From<m64prs_sys::EmuState> for CoreEmuState {
    fn from(value: m64prs_sys::EmuState) -> Self {
        match value {
            m64prs_sys::EmuState::Stopped => Self::Stopped,
            m64prs_sys::EmuState::Running => Self::Running,
            m64prs_sys::EmuState::Paused => Self::Paused,
        }
    }
}
glib_enum_display!(CoreEmuState);