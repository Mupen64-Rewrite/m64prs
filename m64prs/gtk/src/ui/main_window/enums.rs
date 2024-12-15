use glib::prelude::*;
use m64prs_gtk_utils::glib_enum_display;

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
pub enum MainEmuState {
    Uninit = 0,
    Stopped = 1,
    Running = 2,
    Paused = 3
}
impl Default for MainEmuState {
    fn default() -> Self {
        MainEmuState::Uninit
    }
}
impl From<m64prs_sys::EmuState> for MainEmuState {
    fn from(value: m64prs_sys::EmuState) -> Self {
        match value {
            m64prs_sys::EmuState::Stopped => Self::Stopped,
            m64prs_sys::EmuState::Running => Self::Running,
            m64prs_sys::EmuState::Paused => Self::Paused,
        }
    }
}
glib_enum_display!(MainEmuState);