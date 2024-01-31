use std::{env::args, error::Error};

use m64prs_core::{enums::PluginType, Core};



fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().skip(1).collect();

    Core::load("/usr/lib/libmupen64plus.so.2")?;
    let mut core = Core::get_mut();

    core.load_rom(&args[0])?;
    println!("Loaded ROM");

    core.attach_plugin(PluginType::Graphics, "/usr/lib/mupen64plus/mupen64plus-video-rice.so")?;
    core.attach_plugin(PluginType::Audio, "/usr/lib/mupen64plus/mupen64plus-audio-sdl.so")?;
    core.attach_plugin(PluginType::Input, "/usr/lib/mupen64plus/mupen64plus-input-sdl.so")?;
    core.attach_plugin(PluginType::RSP, "/usr/lib/mupen64plus/mupen64plus-rsp-hle.so")?;
    println!("Loaded plugins");

    core.execute_sync()?;

    core.detach_plugin(PluginType::Graphics)?;
    core.detach_plugin(PluginType::Audio)?;
    core.detach_plugin(PluginType::Input)?;
    core.detach_plugin(PluginType::RSP)?;

    Ok(())
}
