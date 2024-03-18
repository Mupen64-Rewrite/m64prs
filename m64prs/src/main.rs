use std::{error::Error, path::PathBuf, sync::Arc, thread, time::Duration};

use async_std::task;
use m64prs_core::{Core, Plugin};

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

    thread::sleep(Duration::from_secs(2));
    println!("Saving state");
    task::block_on(core.save_state()).unwrap();
    println!("State saved");
    thread::sleep(Duration::from_secs(5));
    println!("Loading state");
    task::block_on(core.load_state()).unwrap();
    println!("State loaded");
    thread::sleep(Duration::from_secs(5));
    core.stop().unwrap();

    exec_thread.join().unwrap();
}
