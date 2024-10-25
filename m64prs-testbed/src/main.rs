use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, thread};
use std::{path::PathBuf, sync::Arc};

use std::sync::RwLock;
use m64prs_core::{Core, Plugin};
use m64prs_sys::{Buttons, EmuState};
use vidext::VideoState;

mod vidext;

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

        // #[cfg(any())]
        {
            let (_, inputs) = m64prs_movie::load_m64(&args[3]);
            let mut counter: usize = 0;
            let mut first_frame = true;

            core.set_input_filter(Box::new(move |port, input| {
                if port != 0 {
                    return input;
                }
                if first_frame {
                    first_frame = false;
                    return input;
                }

                if counter < inputs.len() {
                    let result = inputs[counter];
                    counter += 1;
                    result
                }
                else {
                    Buttons::BLANK
                }
            }));
        }
    }


    {
        let core = core.read().unwrap();
        core.execute().unwrap();
    }
}
