use std::{env::args, error::Error};
use m64prs_core::{enums::PluginType, Core, Plugin};



fn main() -> Result<(), Box<dyn Error>> {
    let _args: Vec<String> = args().skip(1).collect();

    let mut core = Core::load("/usr/lib/libmupen64plus.so.2")?;

    core.load_rom(&_args[0])?;
    println!("Loaded ROM");

    core.attach_plugin(PluginType::Video, Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so")?)?;
    core.attach_plugin(PluginType::Audio, Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so")?)?;
    core.attach_plugin(PluginType::Input, Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so")?)?;
    core.attach_plugin(PluginType::RSP, Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so")?)?;
    println!("Loaded plugins");

    core.execute_sync()?;

    core.detach_plugin(PluginType::Video)?;
    core.detach_plugin(PluginType::Audio)?;
    core.detach_plugin(PluginType::Input)?;
    core.detach_plugin(PluginType::RSP)?;

    Ok(())
}
