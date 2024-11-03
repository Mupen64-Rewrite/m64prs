use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, thread};
use std::{path::PathBuf, sync::Arc};

use std::sync::RwLock;
use m64prs_core::{Core, Plugin};
use m64prs_sys::{Buttons, EmuState};
use movie::MovieInputFilter;
use vidext::VideoState;

mod vidext;
mod movie;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let core = Arc::new(RwLock::new(Core::init(PathBuf::from(&args[1])).unwrap()));

    {
        let core_arc = &core;
        let mut core = core.write().unwrap();

        let vidext_instance = VideoState::new(Arc::clone(core_arc));
        core.override_vidext(vidext_instance).unwrap();

        core.open_rom(&fs::read(PathBuf::from(&args[2])).unwrap()).unwrap();
        core.attach_plugins(
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so").unwrap(),
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so").unwrap(),
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so").unwrap(),
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so").unwrap(),
        )
        .unwrap();

        let cfg_sect = core.cfg_open(c"Video-General").unwrap();
        cfg_sect.set(c"ScreenWidth", 960).unwrap();
        cfg_sect.set(c"ScreenHeight", 720).unwrap();

        let input_handler = MovieInputFilter::from_file(PathBuf::from(&args[3]));
        core.set_input_handler(input_handler).unwrap();
    }

    {
        let core = core.read().unwrap();
        core.execute().unwrap();
    }
}
