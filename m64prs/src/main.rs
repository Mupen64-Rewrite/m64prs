use log::info;
use m64prs_core::{ctypes::PluginType, types::VideoExtension, CoreInner, Plugin};
use std::{env::args, error::Error};

mod vidext;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // encode_test()?;

    Ok(())
}

/// quick test function showcasing the rough workflow when using core.
#[allow(dead_code)]
#[cfg(any())]
fn encode_test() -> Result<(), Box<dyn Error>> {
    let _args: Vec<String> = args().skip(1).collect();

    let core_arc = CoreInner::load(&_args[0])?;
    CoreInner::override_vidext::<vidext::VidextState>(&core_arc)?;

    {
        let mut core = core_arc.write().unwrap();

        core.load_rom(&_args[1])?;
        info!("Loaded ROM");

        core.attach_plugin(
            PluginType::GFX,
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so")?,
        )?;
        core.attach_plugin(
            PluginType::AUDIO,
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so")?,
        )?;
        core.attach_plugin(
            PluginType::INPUT,
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so")?,
        )?;
        core.attach_plugin(
            PluginType::RSP,
            Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so")?,
        )?;
        info!("Loaded plugins");
    }

    {
        let core = core_arc.read().unwrap();
        core.execute_sync()?;
    }

    {
        let mut core = core_arc.write().unwrap();
        core.detach_plugin(PluginType::GFX)?;
        core.detach_plugin(PluginType::AUDIO)?;
        core.detach_plugin(PluginType::INPUT)?;
        core.detach_plugin(PluginType::RSP)?;

        core.close_rom()?;
    }

    Ok(())
}
