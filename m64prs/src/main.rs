use std::{path::PathBuf, sync::Arc};


use m64prs_core::{Core, Plugin};

mod vidext;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let mut core = Core::init(PathBuf::from(&args[1])).unwrap();

    core.override_vidext(&vidext::VIDEXT_TABLE).unwrap();

    core.load_rom(PathBuf::from(&args[2])).unwrap();

    core.attach_plugins(
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so").unwrap(),
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so").unwrap(),
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so").unwrap(),
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so").unwrap(),
    )
    .unwrap();

    let core = Arc::new(core);

    core.execute().unwrap();
}
