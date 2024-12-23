
use glib::translate::TryFromGlib;
use m64prs_gtk_utils::glib_enum_display;
use m64prs_vcr::movie::StartType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, glib::Enum)]
#[enum_type(name = "M64PRS_MovieStartType")]
#[repr(u32)]
pub enum MovieStartType {
    #[enum_value(name = "snapshot")]
    Snapshot = StartType::FROM_SNAPSHOT.0 as u32,
    #[enum_value(name = "reset")]
    Reset = StartType::FROM_RESET.0 as u32,
    #[enum_value(name = "eeprom")]
    Eeprom = StartType::FROM_EEPROM.0 as u32,
}

impl Default for MovieStartType {
    fn default() -> Self {
        Self::Snapshot
    }
}

glib_enum_display!(MovieStartType);

impl From<MovieStartType> for StartType {
    fn from(value: MovieStartType) -> Self {
        match value {
            MovieStartType::Snapshot => StartType::FROM_SNAPSHOT,
            MovieStartType::Reset => StartType::FROM_RESET,
            MovieStartType::Eeprom => StartType::FROM_EEPROM,
        }
    }
}

impl TryFrom<StartType> for MovieStartType {
    type Error = ();

    fn try_from(value: StartType) -> Result<Self, Self::Error> {
        match value {
            StartType::FROM_SNAPSHOT => Ok(MovieStartType::Snapshot),
            StartType::FROM_RESET => Ok(MovieStartType::Reset),
            StartType::FROM_EEPROM => Ok(MovieStartType::Eeprom),
            _ => Err(()),
        }
    }
}
