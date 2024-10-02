use std::{io, path::PathBuf, sync::Arc};

use std::sync::RwLock;
use m64prs_core::{Core, Plugin};
use vidext::init_video_state;

mod vidext;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let core = Arc::new(RwLock::new(Core::init(PathBuf::from(&args[1])).unwrap()));
    init_video_state(Arc::clone(&core));

    {
        let mut core = core.write().unwrap();
        core.load_rom(PathBuf::from(&args[2])).unwrap();
        core.attach_plugins(
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-glide64mk2.so").unwrap(),
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so").unwrap(),
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so").unwrap(),
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so").unwrap(),
        )
        .unwrap();
    }


    {
        let core = core.read().unwrap();
        core.execute().unwrap();
    }
}
