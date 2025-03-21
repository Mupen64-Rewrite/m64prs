/*!
Table derived from SDL 2 sources.
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

use m64prs_sys::key::Scancode;

const XF86_KEYSYMS: [Option<Scancode>; 248] = [
    None,                           // NoSymbol
    Some(Scancode::Escape),         // Escape
    Some(Scancode::Num1),           // 1
    Some(Scancode::Num2),           // 2
    Some(Scancode::Num3),           // 3
    Some(Scancode::Num4),           // 4
    Some(Scancode::Num5),           // 5
    Some(Scancode::Num6),           // 6
    Some(Scancode::Num7),           // 7
    Some(Scancode::Num8),           // 8
    Some(Scancode::Num9),           // 9
    Some(Scancode::Num0),           // 0
    Some(Scancode::Minus),          // minus
    Some(Scancode::Equals),         // equal
    Some(Scancode::Backspace),      // BackSpace
    Some(Scancode::Tab),            // Tab
    Some(Scancode::Q),              // q
    Some(Scancode::W),              // w
    Some(Scancode::E),              // e
    Some(Scancode::R),              // r
    Some(Scancode::T),              // t
    Some(Scancode::Y),              // y
    Some(Scancode::U),              // u
    Some(Scancode::I),              // i
    Some(Scancode::O),              // o
    Some(Scancode::P),              // p
    Some(Scancode::LeftBracket),    // bracketleft
    Some(Scancode::RightBracket),   // bracketright
    Some(Scancode::Return),         // Return
    Some(Scancode::LCtrl),          // Control_L
    Some(Scancode::A),              // a
    Some(Scancode::S),              // s
    Some(Scancode::D),              // d
    Some(Scancode::F),              // f
    Some(Scancode::G),              // g
    Some(Scancode::H),              // h
    Some(Scancode::J),              // j
    Some(Scancode::K),              // k
    Some(Scancode::L),              // l
    Some(Scancode::Semicolon),      // semicolon
    Some(Scancode::Apostrophe),     // apostrophe
    Some(Scancode::Grave),          // grave
    Some(Scancode::LShift),         // Shift_L
    Some(Scancode::Backslash),      // backslash
    Some(Scancode::Z),              // z
    Some(Scancode::X),              // x
    Some(Scancode::C),              // c
    Some(Scancode::V),              // v
    Some(Scancode::B),              // b
    Some(Scancode::N),              // n
    Some(Scancode::M),              // m
    Some(Scancode::Comma),          // comma
    Some(Scancode::Period),         // period
    Some(Scancode::Slash),          // slash
    Some(Scancode::RShift),         // Shift_R
    Some(Scancode::KpMultiply),     // KpMultiply
    Some(Scancode::LAlt),           // Alt_L
    Some(Scancode::Space),          // space
    Some(Scancode::CapsLock),       // Caps_Lock
    Some(Scancode::F1),             // F1
    Some(Scancode::F2),             // F2
    Some(Scancode::F3),             // F3
    Some(Scancode::F4),             // F4
    Some(Scancode::F5),             // F5
    Some(Scancode::F6),             // F6
    Some(Scancode::F7),             // F7
    Some(Scancode::F8),             // F8
    Some(Scancode::F9),             // F9
    Some(Scancode::F10),            // F10
    Some(Scancode::NumLockClear),   // Num_Lock
    Some(Scancode::ScrollLock),     // Scroll_Lock
    Some(Scancode::Kp7),            // KpHome
    Some(Scancode::Kp8),            // KpUp
    Some(Scancode::Kp9),            // KpPrior
    Some(Scancode::KpMinus),        // KpSubtract
    Some(Scancode::Kp4),            // KpLeft
    Some(Scancode::Kp5),            // KpBegin
    Some(Scancode::Kp6),            // KpRight
    Some(Scancode::KpPlus),         // KpAdd
    Some(Scancode::Kp1),            // KpEnd
    Some(Scancode::Kp2),            // KpDown
    Some(Scancode::Kp3),            // KpNext
    Some(Scancode::Kp0),            // KpInsert
    Some(Scancode::KpPeriod),       // KpDelete
    Some(Scancode::RAlt),           // ISO_Level3_Shift
    Some(Scancode::Mode),           // ????
    Some(Scancode::NonUsBackslash), // less
    Some(Scancode::F11),            // F11
    Some(Scancode::F12),            // F12
    Some(Scancode::International1), // \_
    Some(Scancode::Lang3),          // Katakana
    Some(Scancode::Lang4),          // Hiragana
    Some(Scancode::International4), // Henkan_Mode
    Some(Scancode::International2), // Hiragana_Katakana
    Some(Scancode::International5), // Muhenkan
    None,                           // NoSymbol
    Some(Scancode::KpEnter),        // KpEnter
    Some(Scancode::RCtrl),          // Control_R
    Some(Scancode::KpDivide),       // KpDivide
    Some(Scancode::PrintScreen),    // Print
    Some(Scancode::RAlt),           // ISO_Level3_Shift, ALTGR, RALT
    None,                           // Linefeed
    Some(Scancode::Home),           // Home
    Some(Scancode::Up),             // Up
    Some(Scancode::PageUp),         // Prior
    Some(Scancode::Left),           // Left
    Some(Scancode::Right),          // Right
    Some(Scancode::End),            // End
    Some(Scancode::Down),           // Down
    Some(Scancode::PageDown),       // Next
    Some(Scancode::Insert),         // Insert
    Some(Scancode::Delete),         // Delete
    None,                           // NoSymbol
    Some(Scancode::Mute),           // XF86AudioMute
    Some(Scancode::VolumeDown),     // XF86AudioLowerVolume
    Some(Scancode::VolumeUp),       // XF86AudioRaiseVolume
    Some(Scancode::Power),          // XF86PowerOff
    Some(Scancode::KpEquals),       // KpEqual
    Some(Scancode::KpPlusMinus),    // plusminus
    Some(Scancode::Pause),          // Pause
    None,                           // XF86LaunchA
    Some(Scancode::KpPeriod),       // KpDecimal
    Some(Scancode::Lang1),          // Hangul
    Some(Scancode::Lang2),          // Hangul_Hanja
    Some(Scancode::International3), // Yen
    Some(Scancode::LGui),           // Super_L
    Some(Scancode::RGui),           // Super_R
    Some(Scancode::Application),    // Menu
    Some(Scancode::Cancel),         // Cancel
    Some(Scancode::Again),          // Redo
    None,                           // SunProps
    Some(Scancode::Undo),           // Undo
    None,                           // SunFront
    Some(Scancode::Copy),           // XF86Copy
    None,                           // SunOpen, XF86Open
    Some(Scancode::Paste),          // XF86Paste
    Some(Scancode::Find),           // Find
    Some(Scancode::Cut),            // XF86Cut
    Some(Scancode::Help),           // Help
    Some(Scancode::Menu),           // XF86MenuKB
    Some(Scancode::Calculator),     // XF86Calculator
    None,                           // NoSymbol
    Some(Scancode::Sleep),          // XF86Sleep
    None,                           // XF86WakeUp
    None,                           // XF86Explorer
    None,                           // XF86Send
    None,                           // NoSymbol
    None,                           // XF86Xfer
    Some(Scancode::App1),           // XF86Launch1
    Some(Scancode::App2),           // XF86Launch2
    Some(Scancode::Www),            // XF86WWW
    None,                           // XF86DOS
    None,                           // XF86ScreenSaver
    None,                           // XF86RotateWindows
    None,                           // XF86TaskPane
    Some(Scancode::Mail),           // XF86Mail
    Some(Scancode::AcBookmarks),    // XF86Favorites
    Some(Scancode::Computer),       // XF86MyComputer
    Some(Scancode::AcBack),         // XF86Back
    Some(Scancode::AcForward),      // XF86Forward
    None,                           // NoSymbol
    Some(Scancode::Eject),          // XF86Eject
    Some(Scancode::Eject),          // XF86Eject
    Some(Scancode::AudioNext),      // XF86AudioNext
    Some(Scancode::AudioPlay),      // XF86AudioPlay
    Some(Scancode::AudioPrev),      // XF86AudioPrev
    Some(Scancode::AudioStop),      // XF86AudioStop
    None,                           // XF86AudioRecord
    None,                           // XF86AudioRewind
    None,                           // XF86Phone
    None,                           // NoSymbol
    Some(Scancode::F13),            // XF86Tools
    Some(Scancode::AcHome),         // XF86HomePage
    Some(Scancode::AcRefresh),      // XF86Reload
    None,                           // XF86Close
    None,                           // NoSymbol
    None,                           // NoSymbol
    None,                           // XF86ScrollUp
    None,                           // XF86ScrollDown
    Some(Scancode::KpLeftParen),    // parenleft
    Some(Scancode::KpRightParen),   // parenright
    None,                           // XF86New
    Some(Scancode::Again),          // Redo
    Some(Scancode::F13),            // XF86Tools
    Some(Scancode::F14),            // XF86Launch5
    Some(Scancode::F15),            // XF86Launch6
    Some(Scancode::F16),            // XF86Launch7
    Some(Scancode::F17),            // XF86Launch8
    Some(Scancode::F18),            // XF86Launch9
    Some(Scancode::F19),            // NoSymbol
    Some(Scancode::F20),            // XF86AudioMicMute
    None,                           // XF86TouchpadToggle
    None,                           // XF86TouchpadOn
    None,                           // XF86TouchpadOff
    None,                           // NoSymbol
    Some(Scancode::Mode),           // Mode_switch
    None,                           // NoSymbol
    None,                           // NoSymbol
    None,                           // NoSymbol
    None,                           // NoSymbol
    Some(Scancode::AudioPlay),      // XF86AudioPlay
    None,                           // XF86AudioPause
    None,                           // XF86Launch3
    None,                           // XF86Launch4
    None,                           // XF86LaunchB
    None,                           // XF86Suspend
    None,                           // XF86Close
    Some(Scancode::AudioPlay),      // XF86AudioPlay
    None,                           // XF86AudioForward
    None,                           // NoSymbol
    Some(Scancode::PrintScreen),    // Print
    None,                           // NoSymbol
    None,                           // XF86WebCam
    None,                           // XF86AudioPreset
    None,                           // NoSymbol
    Some(Scancode::Mail),           // XF86Mail
    None,                           // XF86Messenger
    Some(Scancode::AcSearch),       // XF86Search
    None,                           // XF86Go
    None,                           // XF86Finance
    None,                           // XF86Game
    None,                           // XF86Shop
    None,                           // NoSymbol
    Some(Scancode::Cancel),         // Cancel
    Some(Scancode::BrightnessDown), // XF86MonBrightnessDown
    Some(Scancode::BrightnessUp),   // XF86MonBrightnessUp
    Some(Scancode::MediaSelect),    // XF86AudioMedia
    Some(Scancode::DisplaySwitch),  // XF86Display
    Some(Scancode::KbdIllumToggle), // XF86KbdLightOnOff
    Some(Scancode::KbdIllumDown),   // XF86KbdBrightnessDown
    Some(Scancode::KbdIllumUp),     // XF86KbdBrightnessUp
    None,                           // XF86Send
    None,                           // XF86Reply
    None,                           // XF86MailForward
    None,                           // XF86Save
    None,                           // XF86Documents
    None,                           // XF86Battery
    None,                           // XF86Bluetooth
    None,                           // XF86WLAN
    None,                           // XF86UWB
    None,                           // NoSymbol
    None,                           // XF86Next_VMode
    None,                           // XF86Prev_VMode
    None,                           // XF86MonBrightnessCycle
    None,                           // XF86BrightnessAuto
    None,                           // XF86DisplayOff
    None,                           // XF86WWAN
    None,                           // XF86RFKill
];

pub(super) fn map_native_keycode(key: u32) -> Option<Scancode> {
    // This may be OS-specific but it should work for all up-to-date X11/Wayland systems.
    // Linux keycodes map well to X11 keycodes, and X11 stipulates a minimum of 8 keys.
    //
    // Current implementations leave those first 8 keys blank and start their from index
    // 8 upwards, so generally, (WM key) = (Linux key) + 8.
    match key {
        0..8 => None,
        // Reported keycode = XF86 code + 8
        8.. => XF86_KEYSYMS
            .get((key - 8) as usize)
            .cloned()
            .and_then(|inner| inner),
    }
}
