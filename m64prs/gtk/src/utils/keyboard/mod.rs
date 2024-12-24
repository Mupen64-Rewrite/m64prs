use gdk::prelude::*;
use m64prs_sys::key::{Mod, Scancode};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Emulates SDL's native keyboard mapping.
pub fn into_sdl_scancode(display: &gdk::Display, code: u32) -> Option<Scancode> {
    #[cfg(target_os = "windows")]
    {
        if display.is::<gdk_win32::Win32Display>() {
            return windows::map_native_keycode(code);
        }
        unreachable!()
    }
    #[cfg(target_os = "linux")]
    {
        #[cfg(feature = "x11")]
        if display.is::<gdk_x11::X11Display>() {
            return linux::map_native_keycode(code);
        }
        #[cfg(feature = "wayland")]
        if display.is::<gdk_wayland::WaylandDisplay>() {
            return linux::map_native_keycode(code);
        }
        unreachable!()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    compile_error!("Platform is currently not supported");
}

pub fn into_sdl_modifiers(gdk_mod: gdk::ModifierType) -> Mod {
    let mut sdl_mod = Mod::empty();
    macro_rules! map_bit {
        ($src_bit:expr, $dst_bit:expr) => {
            if gdk_mod.contains($src_bit) {
                sdl_mod |= $dst_bit;
            }
        };
    }

    map_bit!(gdk::ModifierType::CONTROL_MASK, Mod::LCTRLMOD);
    map_bit!(gdk::ModifierType::ALT_MASK, Mod::LALTMOD);
    map_bit!(gdk::ModifierType::SHIFT_MASK, Mod::LSHIFTMOD);
    map_bit!(gdk::ModifierType::SUPER_MASK, Mod::LGUIMOD);

    sdl_mod
}
