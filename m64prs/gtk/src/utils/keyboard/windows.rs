/*!
Scancode translation logic mostly derived from SDL 2, with some changes
to account for the VKey translation GDK already does before handing the code to us.

```text,ignore
Simple DirectMedia Layer
Copyright (C) 1997-2024 Sam Lantinga <slouken@libsdl.org>

This software is provided 'as-is', without any express or implied
warranty.  In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
    claim that you wrote the original software. If you use this software
    in a product, an acknowledgment in the product documentation would be
    appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be
    misrepresented as being the original software.
3. This notice may not be removed or altered from any source distribution.
```
*/

use m64prs_core::key_forward::Scancode;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, MapVirtualKeyExA, MAPVK_VK_TO_VSC, VIRTUAL_KEY, VK_ATTN, VK_BACK,
    VK_BROWSER_BACK, VK_BROWSER_FAVORITES, VK_BROWSER_FORWARD, VK_BROWSER_HOME, VK_BROWSER_REFRESH,
    VK_BROWSER_SEARCH, VK_BROWSER_STOP, VK_CAPITAL, VK_CONTROL, VK_CRSEL, VK_DOWN, VK_EXECUTE,
    VK_EXSEL, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20, VK_F21, VK_F22,
    VK_F23, VK_F24, VK_HELP, VK_LAUNCH_APP1, VK_LAUNCH_APP2, VK_LAUNCH_MAIL,
    VK_LAUNCH_MEDIA_SELECT, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN,
    VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MODECHANGE,
    VK_NUMLOCK, VK_OEM_102, VK_OEM_CLEAR, VK_OEM_NEC_EQUAL, VK_PAUSE, VK_RCONTROL, VK_RIGHT,
    VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SELECT, VK_UP, VK_V, VK_VOLUME_DOWN, VK_VOLUME_MUTE,
    VK_VOLUME_UP,
};

const WIN32_SCANCODE_TABLE: [Option<Scancode>; 128] = [
    None,
    Some(Scancode::Escape),
    Some(Scancode::Num1),
    Some(Scancode::Num2),
    Some(Scancode::Num3),
    Some(Scancode::Num4),
    Some(Scancode::Num5),
    Some(Scancode::Num6),
    Some(Scancode::Num7),
    Some(Scancode::Num8),
    Some(Scancode::Num9),
    Some(Scancode::Num0),
    Some(Scancode::Minus),
    Some(Scancode::Equals),
    Some(Scancode::Backspace),
    Some(Scancode::Tab),
    Some(Scancode::Q),
    Some(Scancode::W),
    Some(Scancode::E),
    Some(Scancode::R),
    Some(Scancode::T),
    Some(Scancode::Y),
    Some(Scancode::U),
    Some(Scancode::I),
    Some(Scancode::O),
    Some(Scancode::P),
    Some(Scancode::LeftBracket),
    Some(Scancode::RightBracket),
    Some(Scancode::Return),
    Some(Scancode::LCtrl),
    Some(Scancode::A),
    Some(Scancode::S),
    Some(Scancode::D),
    Some(Scancode::F),
    Some(Scancode::G),
    Some(Scancode::H),
    Some(Scancode::J),
    Some(Scancode::K),
    Some(Scancode::L),
    Some(Scancode::Semicolon),
    Some(Scancode::Apostrophe),
    Some(Scancode::Grave),
    Some(Scancode::LShift),
    Some(Scancode::Backslash),
    Some(Scancode::Z),
    Some(Scancode::X),
    Some(Scancode::C),
    Some(Scancode::V),
    Some(Scancode::B),
    Some(Scancode::N),
    Some(Scancode::M),
    Some(Scancode::Comma),
    Some(Scancode::Period),
    Some(Scancode::Slash),
    Some(Scancode::RShift),
    Some(Scancode::PrintScreen),
    Some(Scancode::LAlt),
    Some(Scancode::Space),
    Some(Scancode::CapsLock),
    Some(Scancode::F1),
    Some(Scancode::F2),
    Some(Scancode::F3),
    Some(Scancode::F4),
    Some(Scancode::F5),
    Some(Scancode::F6),
    Some(Scancode::F7),
    Some(Scancode::F8),
    Some(Scancode::F9),
    Some(Scancode::F10),
    Some(Scancode::NumLockClear),
    Some(Scancode::ScrollLock),
    Some(Scancode::Home),
    Some(Scancode::Up),
    Some(Scancode::PageUp),
    Some(Scancode::KpMinus),
    Some(Scancode::Left),
    Some(Scancode::Kp5),
    Some(Scancode::Right),
    Some(Scancode::KpPlus),
    Some(Scancode::End),
    Some(Scancode::Down),
    Some(Scancode::PageDown),
    Some(Scancode::Insert),
    Some(Scancode::Delete),
    None,
    None,
    Some(Scancode::NonUsBackslash),
    Some(Scancode::F11),
    Some(Scancode::F12),
    Some(Scancode::Pause),
    None,
    Some(Scancode::LGui),
    Some(Scancode::RGui),
    Some(Scancode::Application),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(Scancode::F13),
    Some(Scancode::F14),
    Some(Scancode::F15),
    Some(Scancode::F16),
    Some(Scancode::F17),
    Some(Scancode::F18),
    Some(Scancode::F19),
    None,
    None,
    None,
    None,
    None,
    Some(Scancode::International2),
    None,
    None,
    Some(Scancode::International1),
    None,
    None,
    None,
    None,
    None,
    Some(Scancode::International4),
    None,
    Some(Scancode::International5),
    None,
    Some(Scancode::International3),
    None,
    None,
];

fn vkey_to_scancode(vk: u32) -> Option<Scancode> {
    match VIRTUAL_KEY(vk as u16) {
        VK_BACK => Some(Scancode::Backspace),
        VK_CAPITAL => Some(Scancode::CapsLock),

        VK_MODECHANGE => Some(Scancode::Mode),
        VK_SELECT => Some(Scancode::Select),
        VK_EXECUTE => Some(Scancode::Execute),
        VK_HELP => Some(Scancode::Help),
        VK_PAUSE => Some(Scancode::Pause),
        VK_NUMLOCK => Some(Scancode::NumLockClear),

        VK_F13 => Some(Scancode::F13),
        VK_F14 => Some(Scancode::F14),
        VK_F15 => Some(Scancode::F15),
        VK_F16 => Some(Scancode::F16),
        VK_F17 => Some(Scancode::F17),
        VK_F18 => Some(Scancode::F18),
        VK_F19 => Some(Scancode::F19),
        VK_F20 => Some(Scancode::F20),
        VK_F21 => Some(Scancode::F21),
        VK_F22 => Some(Scancode::F22),
        VK_F23 => Some(Scancode::F23),
        VK_F24 => Some(Scancode::F24),

        VK_OEM_NEC_EQUAL => Some(Scancode::KpEquals),
        VK_BROWSER_BACK => Some(Scancode::AcBack),
        VK_BROWSER_FORWARD => Some(Scancode::AcForward),
        VK_BROWSER_REFRESH => Some(Scancode::AcRefresh),
        VK_BROWSER_STOP => Some(Scancode::AcStop),
        VK_BROWSER_SEARCH => Some(Scancode::AcSearch),
        VK_BROWSER_FAVORITES => Some(Scancode::AcBookmarks),
        VK_BROWSER_HOME => Some(Scancode::AcHome),
        VK_VOLUME_MUTE => Some(Scancode::Mute),
        VK_VOLUME_DOWN => Some(Scancode::VolumeDown),
        VK_VOLUME_UP => Some(Scancode::VolumeUp),

        VK_MEDIA_NEXT_TRACK => Some(Scancode::AudioNext),
        VK_MEDIA_PREV_TRACK => Some(Scancode::AudioPrev),
        VK_MEDIA_STOP => Some(Scancode::AudioStop),
        VK_MEDIA_PLAY_PAUSE => Some(Scancode::AudioPlay),
        VK_LAUNCH_MAIL => Some(Scancode::Mail),
        VK_LAUNCH_MEDIA_SELECT => Some(Scancode::MediaSelect),

        VK_OEM_102 => Some(Scancode::NonUsBackslash),

        VK_ATTN => Some(Scancode::SysReq),
        VK_CRSEL => Some(Scancode::CrSel),
        VK_EXSEL => Some(Scancode::ExSel),
        VK_OEM_CLEAR => Some(Scancode::Clear),

        VK_LAUNCH_APP1 => Some(Scancode::App1),
        VK_LAUNCH_APP2 => Some(Scancode::App2),

        VK_LSHIFT => Some(Scancode::LShift),
        VK_RSHIFT => Some(Scancode::RShift),
        VK_LMENU => Some(Scancode::LAlt),
        VK_RMENU => Some(Scancode::RAlt),
        VK_LCONTROL => Some(Scancode::LCtrl),
        VK_RCONTROL => Some(Scancode::RCtrl),
        VK_LWIN => Some(Scancode::LGui),
        VK_RWIN => Some(Scancode::RGui),

        _ => None,
    }
}

fn vkey_to_scancode_fallback(key: u32) -> Option<Scancode> {
    match VIRTUAL_KEY(key as u16) {
        VK_LEFT => Some(Scancode::Left),
        VK_UP => Some(Scancode::Up),
        VK_RIGHT => Some(Scancode::Right),
        VK_DOWN => Some(Scancode::Down),
        VK_CONTROL => Some(Scancode::LCtrl),
        VK_V => Some(Scancode::V),
        _ => None,
    }
}

pub(super) fn map_native_keycode(key: u32) -> Option<Scancode> {
    if let Some(scancode) = vkey_to_scancode(key) {
        return Some(scancode);
    }

    let win32_scancode = unsafe {
        let layout = GetKeyboardLayout(0);
        MapVirtualKeyExA(key, MAPVK_VK_TO_VSC, layout)
    };
    if let Some(scancode) = WIN32_SCANCODE_TABLE
        .get(win32_scancode as usize)
        .and_then(|inner| inner.clone())
    {
        return Some(scancode);
    }

    if let Some(scancode) = vkey_to_scancode_fallback(key) {
        return Some(scancode);
    }

    None
}
