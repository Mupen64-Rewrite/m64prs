use std::fs;
use std::{path::PathBuf, sync::Arc};

use std::sync::RwLock;
use m64prs_core::{Core, Plugin};
use vidext::init_video_state;

mod vidext;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let core = Arc::new(RwLock::new(Core::init(PathBuf::from(&args[1])).unwrap()));

    {
        let mut core = core.write().unwrap();
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
    }


    {
        let core = core.read().unwrap();
        core.execute().unwrap();
    }
}
