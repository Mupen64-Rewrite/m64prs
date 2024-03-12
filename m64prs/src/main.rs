use std::{env::args, error::Error};
use m64prs_core::{Core, Plugin, ctypes::PluginType};


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    encode_test()?;
    Ok(())
}

/// quick test function showcasing the rough workflow when using core.
#[allow(dead_code)]
fn encode_test() -> Result<(), Box<dyn Error>> {

    let _args: Vec<String> = args().skip(1).collect();

    let mut core = Core::load(&_args[0])?;

    core.load_rom(&_args[1])?;
    println!("Loaded ROM");

    core.attach_plugin(PluginType::GFX, Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so")?)?;
    core.attach_plugin(PluginType::AUDIO, Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so")?)?;
    core.attach_plugin(PluginType::INPUT, Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so")?)?;
    core.attach_plugin(PluginType::RSP, Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so")?)?;
    println!("Loaded plugins");

    core.execute_sync()?;

    core.detach_plugin(PluginType::GFX)?;
    core.detach_plugin(PluginType::AUDIO)?;
    core.detach_plugin(PluginType::INPUT)?;
    core.detach_plugin(PluginType::RSP)?;

    core.close_rom()?;

    Ok(())
}