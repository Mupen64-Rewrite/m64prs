use std::{error::Error, path::PathBuf, sync::Arc, thread, time::Duration};

use async_std::task;
use m64prs_core::{Core, Plugin};

mod vidext;

fn main() {
    ::env_logger::init();

    let args: Vec<_> = std::env::args().collect();

    let mut core = Core::init(PathBuf::from(&args[1])).unwrap();

    core.load_rom(PathBuf::from(&args[2])).unwrap();

    core.attach_plugins(
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so").unwrap(),
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so").unwrap(),
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so").unwrap(),
        Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so").unwrap(),
    ).unwrap();

    let core = Arc::new(core);

    let exec_thread = {
        let core = Arc::clone(&core);
        thread::spawn(move || {
            core.execute().unwrap();
        })
    };

    task::block_on(async {
        task::sleep(Duration::from_secs(2)).await;

        let fut1 = core.save_state();
        let fut2 = core.load_state();

        let (res1, res2) = futures::join!(fut1, fut2);
        res1.unwrap();
        res2.unwrap();
    });

    exec_thread.join().unwrap();
}
