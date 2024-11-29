use std::{path::PathBuf, sync::Mutex};

use m64prs_core::tas_callbacks::SaveHandler;
use m64prs_movie::M64Header;
use m64prs_sys::Buttons;

pub struct VcrState(Mutex<Option<VcrStateInner>>);

struct VcrStateInner {
    path: PathBuf,
    header: M64Header,
    inputs: Vec<Buttons>,
    read_only: bool,
}