use std::{fs, thread};
use std::{path::PathBuf, sync::Arc};

use m64prs_core::plugin::PluginSet;
use m64prs_core::{Core, Plugin};
use movie::MovieInputFilter;
use std::sync::RwLock;

mod movie;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let core = Arc::new(RwLock::new(Core::init(PathBuf::from(&args[1])).unwrap()));

    {
        let mut core = core.write().unwrap();

        // let vidext_instance = VideoState::new(Arc::clone(core_arc));
        // core.override_vidext(vidext_instance).unwrap();

        core.open_rom(&fs::read(PathBuf::from(&args[2])).unwrap())
            .unwrap();
        core.attach_plugins(PluginSet {
            graphics: Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so").unwrap(),
            audio: Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so").unwrap(),
            input: Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so").unwrap(),
            rsp: Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so").unwrap(),
        })
        .unwrap();

        let mut cfg_sect = core.cfg_open(c"Video-General").unwrap();
        cfg_sect.set(c"ScreenWidth", 960).unwrap();
        cfg_sect.set(c"ScreenHeight", 720).unwrap();
        std::mem::drop(cfg_sect);

        let input_handler = MovieInputFilter::from_file(PathBuf::from(&args[3]));
        core.set_input_handler(input_handler).unwrap();
    }

    let core_thread = {
        let core = Arc::clone(&core);
        thread::spawn(move || {
            core.read().unwrap().execute().unwrap();
        })
    };

    core_thread.join().unwrap();
}
