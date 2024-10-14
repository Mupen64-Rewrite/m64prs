use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, thread};
use std::{path::PathBuf, sync::Arc};

use std::sync::RwLock;
use m64prs_core::{Core, Plugin};
use m64prs_sys::{Buttons, EmuState};

mod vidext;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let core = Arc::new(RwLock::new(Core::init(PathBuf::from(&args[1])).unwrap()));
    vidext::init_video_state(Arc::clone(&core));

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

        // #[cfg(any())]
        {
            let (_, inputs) = m64prs_movie::load_m64(&args[3]);
            let mut counter: usize = 0;

            core.set_input_filter(Box::new(move |port, input| {
                if port != 0 {
                    return input;
                }
                const OFFSET: usize = 1;

                if counter < inputs.len() + OFFSET {
                    let result = match counter {
                        OFFSET.. => inputs[counter - OFFSET],
                        0.. => Buttons::BLANK
                    };
                    println!("{:4}: {:?}", counter, result);
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
