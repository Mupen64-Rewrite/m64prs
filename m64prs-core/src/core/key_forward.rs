use m64prs_sys::Command;

use crate::error::M64PError;

use super::Core;

pub use enums::*;

impl Core {
    pub fn forward_key_down(
        &self,
        sdl_key: Option<Keycode>,
        sdl_mod: Mod,
    ) -> Result<(), M64PError> {
        let sdl_key = sdl_key.map_or(0, |key| key.into_i32()) as u16 as u32;
        let sdl_mod = sdl_mod.bits() as u32;

        let int_param = ((sdl_mod << 16) | sdl_key) as i32;

        self.do_command_i(Command::SendSdlKeydown, int_param)
    }
    pub fn forward_key_up(&self, sdl_key: Option<Keycode>, sdl_mod: Mod) -> Result<(), M64PError> {
        let sdl_key = sdl_key.map_or(0, |key| key.into_i32()) as u16 as u32;
        let sdl_mod = sdl_mod.bits() as u32;

        let int_param = ((sdl_mod << 16) | sdl_key) as i32;

        self.do_command_i(Command::SendSdlKeyup, int_param)
    }
}
#[cfg(not(feature = "sdl2"))]
mod enums {
    //! Enums derived from the [sdl2](https://github.com/Rust-SDL2/rust-sdl2) crate.
    //!
    //! ```text,ignore
    //! The MIT License (MIT)
    //!
    //! Copyright (c) 2013 Mozilla Foundation
    //!
    //! Permission is hereby granted, free of charge, to any person obtaining a copy of
    //! this software and associated documentation files (the "Software"), to deal in
    //! the Software without restriction, including without limitation the rights to
    //! use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
    //! the Software, and to permit persons to whom the Software is furnished to do so,
    //! subject to the following conditions:
    //!
    //! The above copyright notice and this permission notice shall be included in all
    //! copies or substantial portions of the Software.
    //!
    //! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    //! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    //! FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
    //! COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
    //! IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
    //! CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
    //! ```

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Keycode(i32);

    impl Keycode {
        pub fn into_i32(self) -> i32 {
            self.0
        }
        pub fn from_i32(value: i32) -> Self {
            Self(value)
        }
    }

    #[allow(non_upper_case_globals)]
    impl Keycode {
        pub const BACKSPACE: Keycode = Keycode(SDL_KeyCode::SDLK_BACKSPACE as i32);
        pub const TAB: Keycode = Keycode(SDL_KeyCode::SDLK_TAB as i32);
        pub const RETURN: Keycode = Keycode(SDL_KeyCode::SDLK_RETURN as i32);
        pub const ESCAPE: Keycode = Keycode(SDL_KeyCode::SDLK_ESCAPE as i32);
        pub const SPACE: Keycode = Keycode(SDL_KeyCode::SDLK_SPACE as i32);
        pub const EXCLAIM: Keycode = Keycode(SDL_KeyCode::SDLK_EXCLAIM as i32);
        pub const QUOTEDBL: Keycode = Keycode(SDL_KeyCode::SDLK_QUOTEDBL as i32);
        pub const HASH: Keycode = Keycode(SDL_KeyCode::SDLK_HASH as i32);
        pub const DOLLAR: Keycode = Keycode(SDL_KeyCode::SDLK_DOLLAR as i32);
        pub const PERCENT: Keycode = Keycode(SDL_KeyCode::SDLK_PERCENT as i32);
        pub const AMPERSAND: Keycode = Keycode(SDL_KeyCode::SDLK_AMPERSAND as i32);
        pub const QUOTE: Keycode = Keycode(SDL_KeyCode::SDLK_QUOTE as i32);
        pub const LEFTPAREN: Keycode = Keycode(SDL_KeyCode::SDLK_LEFTPAREN as i32);
        pub const RIGHTPAREN: Keycode = Keycode(SDL_KeyCode::SDLK_RIGHTPAREN as i32);
        pub const ASTERISK: Keycode = Keycode(SDL_KeyCode::SDLK_ASTERISK as i32);
        pub const PLUS: Keycode = Keycode(SDL_KeyCode::SDLK_PLUS as i32);
        pub const COMMA: Keycode = Keycode(SDL_KeyCode::SDLK_COMMA as i32);
        pub const MINUS: Keycode = Keycode(SDL_KeyCode::SDLK_MINUS as i32);
        pub const PERIOD: Keycode = Keycode(SDL_KeyCode::SDLK_PERIOD as i32);
        pub const SLASH: Keycode = Keycode(SDL_KeyCode::SDLK_SLASH as i32);
        pub const NUM_0: Keycode = Keycode(SDL_KeyCode::SDLK_0 as i32);
        pub const NUM_1: Keycode = Keycode(SDL_KeyCode::SDLK_1 as i32);
        pub const NUM_2: Keycode = Keycode(SDL_KeyCode::SDLK_2 as i32);
        pub const NUM_3: Keycode = Keycode(SDL_KeyCode::SDLK_3 as i32);
        pub const NUM_4: Keycode = Keycode(SDL_KeyCode::SDLK_4 as i32);
        pub const NUM_5: Keycode = Keycode(SDL_KeyCode::SDLK_5 as i32);
        pub const NUM_6: Keycode = Keycode(SDL_KeyCode::SDLK_6 as i32);
        pub const NUM_7: Keycode = Keycode(SDL_KeyCode::SDLK_7 as i32);
        pub const NUM_8: Keycode = Keycode(SDL_KeyCode::SDLK_8 as i32);
        pub const NUM_9: Keycode = Keycode(SDL_KeyCode::SDLK_9 as i32);
        pub const COLON: Keycode = Keycode(SDL_KeyCode::SDLK_COLON as i32);
        pub const SEMICOLON: Keycode = Keycode(SDL_KeyCode::SDLK_SEMICOLON as i32);
        pub const LESS: Keycode = Keycode(SDL_KeyCode::SDLK_LESS as i32);
        pub const EQUALS: Keycode = Keycode(SDL_KeyCode::SDLK_EQUALS as i32);
        pub const GREATER: Keycode = Keycode(SDL_KeyCode::SDLK_GREATER as i32);
        pub const QUESTION: Keycode = Keycode(SDL_KeyCode::SDLK_QUESTION as i32);
        pub const AT: Keycode = Keycode(SDL_KeyCode::SDLK_AT as i32);
        pub const LEFTBRACKET: Keycode = Keycode(SDL_KeyCode::SDLK_LEFTBRACKET as i32);
        pub const BACKSLASH: Keycode = Keycode(SDL_KeyCode::SDLK_BACKSLASH as i32);
        pub const RIGHTBRACKET: Keycode = Keycode(SDL_KeyCode::SDLK_RIGHTBRACKET as i32);
        pub const CARET: Keycode = Keycode(SDL_KeyCode::SDLK_CARET as i32);
        pub const UNDERSCORE: Keycode = Keycode(SDL_KeyCode::SDLK_UNDERSCORE as i32);
        pub const BACKQUOTE: Keycode = Keycode(SDL_KeyCode::SDLK_BACKQUOTE as i32);
        pub const A: Keycode = Keycode(SDL_KeyCode::SDLK_a as i32);
        pub const B: Keycode = Keycode(SDL_KeyCode::SDLK_b as i32);
        pub const C: Keycode = Keycode(SDL_KeyCode::SDLK_c as i32);
        pub const D: Keycode = Keycode(SDL_KeyCode::SDLK_d as i32);
        pub const E: Keycode = Keycode(SDL_KeyCode::SDLK_e as i32);
        pub const F: Keycode = Keycode(SDL_KeyCode::SDLK_f as i32);
        pub const G: Keycode = Keycode(SDL_KeyCode::SDLK_g as i32);
        pub const H: Keycode = Keycode(SDL_KeyCode::SDLK_h as i32);
        pub const I: Keycode = Keycode(SDL_KeyCode::SDLK_i as i32);
        pub const J: Keycode = Keycode(SDL_KeyCode::SDLK_j as i32);
        pub const K: Keycode = Keycode(SDL_KeyCode::SDLK_k as i32);
        pub const L: Keycode = Keycode(SDL_KeyCode::SDLK_l as i32);
        pub const M: Keycode = Keycode(SDL_KeyCode::SDLK_m as i32);
        pub const N: Keycode = Keycode(SDL_KeyCode::SDLK_n as i32);
        pub const O: Keycode = Keycode(SDL_KeyCode::SDLK_o as i32);
        pub const P: Keycode = Keycode(SDL_KeyCode::SDLK_p as i32);
        pub const Q: Keycode = Keycode(SDL_KeyCode::SDLK_q as i32);
        pub const R: Keycode = Keycode(SDL_KeyCode::SDLK_r as i32);
        pub const S: Keycode = Keycode(SDL_KeyCode::SDLK_s as i32);
        pub const T: Keycode = Keycode(SDL_KeyCode::SDLK_t as i32);
        pub const U: Keycode = Keycode(SDL_KeyCode::SDLK_u as i32);
        pub const V: Keycode = Keycode(SDL_KeyCode::SDLK_v as i32);
        pub const W: Keycode = Keycode(SDL_KeyCode::SDLK_w as i32);
        pub const X: Keycode = Keycode(SDL_KeyCode::SDLK_x as i32);
        pub const Y: Keycode = Keycode(SDL_KeyCode::SDLK_y as i32);
        pub const Z: Keycode = Keycode(SDL_KeyCode::SDLK_z as i32);
        pub const DELETE: Keycode = Keycode(SDL_KeyCode::SDLK_DELETE as i32);
        pub const CAPSLOCK: Keycode = Keycode(SDL_KeyCode::SDLK_CAPSLOCK as i32);
        pub const F1: Keycode = Keycode(SDL_KeyCode::SDLK_F1 as i32);
        pub const F2: Keycode = Keycode(SDL_KeyCode::SDLK_F2 as i32);
        pub const F3: Keycode = Keycode(SDL_KeyCode::SDLK_F3 as i32);
        pub const F4: Keycode = Keycode(SDL_KeyCode::SDLK_F4 as i32);
        pub const F5: Keycode = Keycode(SDL_KeyCode::SDLK_F5 as i32);
        pub const F6: Keycode = Keycode(SDL_KeyCode::SDLK_F6 as i32);
        pub const F7: Keycode = Keycode(SDL_KeyCode::SDLK_F7 as i32);
        pub const F8: Keycode = Keycode(SDL_KeyCode::SDLK_F8 as i32);
        pub const F9: Keycode = Keycode(SDL_KeyCode::SDLK_F9 as i32);
        pub const F10: Keycode = Keycode(SDL_KeyCode::SDLK_F10 as i32);
        pub const F11: Keycode = Keycode(SDL_KeyCode::SDLK_F11 as i32);
        pub const F12: Keycode = Keycode(SDL_KeyCode::SDLK_F12 as i32);
        pub const PRINTSCREEN: Keycode = Keycode(SDL_KeyCode::SDLK_PRINTSCREEN as i32);
        pub const SCROLLLOCK: Keycode = Keycode(SDL_KeyCode::SDLK_SCROLLLOCK as i32);
        pub const PAUSE: Keycode = Keycode(SDL_KeyCode::SDLK_PAUSE as i32);
        pub const INSERT: Keycode = Keycode(SDL_KeyCode::SDLK_INSERT as i32);
        pub const HOME: Keycode = Keycode(SDL_KeyCode::SDLK_HOME as i32);
        pub const PAGEUP: Keycode = Keycode(SDL_KeyCode::SDLK_PAGEUP as i32);
        pub const END: Keycode = Keycode(SDL_KeyCode::SDLK_END as i32);
        pub const PAGEDOWN: Keycode = Keycode(SDL_KeyCode::SDLK_PAGEDOWN as i32);
        pub const RIGHT: Keycode = Keycode(SDL_KeyCode::SDLK_RIGHT as i32);
        pub const LEFT: Keycode = Keycode(SDL_KeyCode::SDLK_LEFT as i32);
        pub const DOWN: Keycode = Keycode(SDL_KeyCode::SDLK_DOWN as i32);
        pub const UP: Keycode = Keycode(SDL_KeyCode::SDLK_UP as i32);
        pub const NUMLOCKCLEAR: Keycode = Keycode(SDL_KeyCode::SDLK_NUMLOCKCLEAR as i32);
        pub const KP_DIVIDE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_DIVIDE as i32);
        pub const KP_MULTIPLY: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MULTIPLY as i32);
        pub const KP_MINUS: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MINUS as i32);
        pub const KP_PLUS: Keycode = Keycode(SDL_KeyCode::SDLK_KP_PLUS as i32);
        pub const KP_ENTER: Keycode = Keycode(SDL_KeyCode::SDLK_KP_ENTER as i32);
        pub const KP_1: Keycode = Keycode(SDL_KeyCode::SDLK_KP_1 as i32);
        pub const KP_2: Keycode = Keycode(SDL_KeyCode::SDLK_KP_2 as i32);
        pub const KP_3: Keycode = Keycode(SDL_KeyCode::SDLK_KP_3 as i32);
        pub const KP_4: Keycode = Keycode(SDL_KeyCode::SDLK_KP_4 as i32);
        pub const KP_5: Keycode = Keycode(SDL_KeyCode::SDLK_KP_5 as i32);
        pub const KP_6: Keycode = Keycode(SDL_KeyCode::SDLK_KP_6 as i32);
        pub const KP_7: Keycode = Keycode(SDL_KeyCode::SDLK_KP_7 as i32);
        pub const KP_8: Keycode = Keycode(SDL_KeyCode::SDLK_KP_8 as i32);
        pub const KP_9: Keycode = Keycode(SDL_KeyCode::SDLK_KP_9 as i32);
        pub const KP_0: Keycode = Keycode(SDL_KeyCode::SDLK_KP_0 as i32);
        pub const KP_PERIOD: Keycode = Keycode(SDL_KeyCode::SDLK_KP_PERIOD as i32);
        pub const APPLICATION: Keycode = Keycode(SDL_KeyCode::SDLK_APPLICATION as i32);
        pub const POWER: Keycode = Keycode(SDL_KeyCode::SDLK_POWER as i32);
        pub const KP_EQUALS: Keycode = Keycode(SDL_KeyCode::SDLK_KP_EQUALS as i32);
        pub const F13: Keycode = Keycode(SDL_KeyCode::SDLK_F13 as i32);
        pub const F14: Keycode = Keycode(SDL_KeyCode::SDLK_F14 as i32);
        pub const F15: Keycode = Keycode(SDL_KeyCode::SDLK_F15 as i32);
        pub const F16: Keycode = Keycode(SDL_KeyCode::SDLK_F16 as i32);
        pub const F17: Keycode = Keycode(SDL_KeyCode::SDLK_F17 as i32);
        pub const F18: Keycode = Keycode(SDL_KeyCode::SDLK_F18 as i32);
        pub const F19: Keycode = Keycode(SDL_KeyCode::SDLK_F19 as i32);
        pub const F20: Keycode = Keycode(SDL_KeyCode::SDLK_F20 as i32);
        pub const F21: Keycode = Keycode(SDL_KeyCode::SDLK_F21 as i32);
        pub const F22: Keycode = Keycode(SDL_KeyCode::SDLK_F22 as i32);
        pub const F23: Keycode = Keycode(SDL_KeyCode::SDLK_F23 as i32);
        pub const F24: Keycode = Keycode(SDL_KeyCode::SDLK_F24 as i32);
        pub const EXECUTE: Keycode = Keycode(SDL_KeyCode::SDLK_EXECUTE as i32);
        pub const HELP: Keycode = Keycode(SDL_KeyCode::SDLK_HELP as i32);
        pub const MENU: Keycode = Keycode(SDL_KeyCode::SDLK_MENU as i32);
        pub const SELECT: Keycode = Keycode(SDL_KeyCode::SDLK_SELECT as i32);
        pub const STOP: Keycode = Keycode(SDL_KeyCode::SDLK_STOP as i32);
        pub const AGAIN: Keycode = Keycode(SDL_KeyCode::SDLK_AGAIN as i32);
        pub const UNDO: Keycode = Keycode(SDL_KeyCode::SDLK_UNDO as i32);
        pub const CUT: Keycode = Keycode(SDL_KeyCode::SDLK_CUT as i32);
        pub const COPY: Keycode = Keycode(SDL_KeyCode::SDLK_COPY as i32);
        pub const PASTE: Keycode = Keycode(SDL_KeyCode::SDLK_PASTE as i32);
        pub const FIND: Keycode = Keycode(SDL_KeyCode::SDLK_FIND as i32);
        pub const MUTE: Keycode = Keycode(SDL_KeyCode::SDLK_MUTE as i32);
        pub const VOLUMEUP: Keycode = Keycode(SDL_KeyCode::SDLK_VOLUMEUP as i32);
        pub const VOLUMEDOWN: Keycode = Keycode(SDL_KeyCode::SDLK_VOLUMEDOWN as i32);
        pub const KP_COMMA: Keycode = Keycode(SDL_KeyCode::SDLK_KP_COMMA as i32);
        pub const KP_EQUALSAS400: Keycode = Keycode(SDL_KeyCode::SDLK_KP_EQUALSAS400 as i32);
        pub const ALTERASE: Keycode = Keycode(SDL_KeyCode::SDLK_ALTERASE as i32);
        pub const SYSREQ: Keycode = Keycode(SDL_KeyCode::SDLK_SYSREQ as i32);
        pub const CANCEL: Keycode = Keycode(SDL_KeyCode::SDLK_CANCEL as i32);
        pub const CLEAR: Keycode = Keycode(SDL_KeyCode::SDLK_CLEAR as i32);
        pub const PRIOR: Keycode = Keycode(SDL_KeyCode::SDLK_PRIOR as i32);
        pub const RETURN2: Keycode = Keycode(SDL_KeyCode::SDLK_RETURN2 as i32);
        pub const SEPARATOR: Keycode = Keycode(SDL_KeyCode::SDLK_SEPARATOR as i32);
        pub const OUT: Keycode = Keycode(SDL_KeyCode::SDLK_OUT as i32);
        pub const OPER: Keycode = Keycode(SDL_KeyCode::SDLK_OPER as i32);
        pub const CLEARAGAIN: Keycode = Keycode(SDL_KeyCode::SDLK_CLEARAGAIN as i32);
        pub const CRSEL: Keycode = Keycode(SDL_KeyCode::SDLK_CRSEL as i32);
        pub const EXSEL: Keycode = Keycode(SDL_KeyCode::SDLK_EXSEL as i32);
        pub const KP_00: Keycode = Keycode(SDL_KeyCode::SDLK_KP_00 as i32);
        pub const KP_000: Keycode = Keycode(SDL_KeyCode::SDLK_KP_000 as i32);
        pub const THOUSANDSSEPARATOR: Keycode =
            Keycode(SDL_KeyCode::SDLK_THOUSANDSSEPARATOR as i32);
        pub const DECIMALSEPARATOR: Keycode = Keycode(SDL_KeyCode::SDLK_DECIMALSEPARATOR as i32);
        pub const CURRENCYUNIT: Keycode = Keycode(SDL_KeyCode::SDLK_CURRENCYUNIT as i32);
        pub const CURRENCYSUBUNIT: Keycode = Keycode(SDL_KeyCode::SDLK_CURRENCYSUBUNIT as i32);
        pub const KP_LEFTPAREN: Keycode = Keycode(SDL_KeyCode::SDLK_KP_LEFTPAREN as i32);
        pub const KP_RIGHTPAREN: Keycode = Keycode(SDL_KeyCode::SDLK_KP_RIGHTPAREN as i32);
        pub const KP_LEFTBRACE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_LEFTBRACE as i32);
        pub const KP_RIGHTBRACE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_RIGHTBRACE as i32);
        pub const KP_TAB: Keycode = Keycode(SDL_KeyCode::SDLK_KP_TAB as i32);
        pub const KP_BACKSPACE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_BACKSPACE as i32);
        pub const KP_A: Keycode = Keycode(SDL_KeyCode::SDLK_KP_A as i32);
        pub const KP_B: Keycode = Keycode(SDL_KeyCode::SDLK_KP_B as i32);
        pub const KP_C: Keycode = Keycode(SDL_KeyCode::SDLK_KP_C as i32);
        pub const KP_D: Keycode = Keycode(SDL_KeyCode::SDLK_KP_D as i32);
        pub const KP_E: Keycode = Keycode(SDL_KeyCode::SDLK_KP_E as i32);
        pub const KP_F: Keycode = Keycode(SDL_KeyCode::SDLK_KP_F as i32);
        pub const KP_XOR: Keycode = Keycode(SDL_KeyCode::SDLK_KP_XOR as i32);
        pub const KP_POWER: Keycode = Keycode(SDL_KeyCode::SDLK_KP_POWER as i32);
        pub const KP_PERCENT: Keycode = Keycode(SDL_KeyCode::SDLK_KP_PERCENT as i32);
        pub const KP_LESS: Keycode = Keycode(SDL_KeyCode::SDLK_KP_LESS as i32);
        pub const KP_GREATER: Keycode = Keycode(SDL_KeyCode::SDLK_KP_GREATER as i32);
        pub const KP_AMPERSAND: Keycode = Keycode(SDL_KeyCode::SDLK_KP_AMPERSAND as i32);
        pub const KP_DBLAMPERSAND: Keycode = Keycode(SDL_KeyCode::SDLK_KP_DBLAMPERSAND as i32);
        pub const KP_VERTICALBAR: Keycode = Keycode(SDL_KeyCode::SDLK_KP_VERTICALBAR as i32);
        pub const KP_DBLVERTICALBAR: Keycode = Keycode(SDL_KeyCode::SDLK_KP_DBLVERTICALBAR as i32);
        pub const KP_COLON: Keycode = Keycode(SDL_KeyCode::SDLK_KP_COLON as i32);
        pub const KP_HASH: Keycode = Keycode(SDL_KeyCode::SDLK_KP_HASH as i32);
        pub const KP_SPACE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_SPACE as i32);
        pub const KP_AT: Keycode = Keycode(SDL_KeyCode::SDLK_KP_AT as i32);
        pub const KP_EXCLAM: Keycode = Keycode(SDL_KeyCode::SDLK_KP_EXCLAM as i32);
        pub const KP_MEMSTORE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMSTORE as i32);
        pub const KP_MEMRECALL: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMRECALL as i32);
        pub const KP_MEMCLEAR: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMCLEAR as i32);
        pub const KP_MEMADD: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMADD as i32);
        pub const KP_MEMSUBTRACT: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMSUBTRACT as i32);
        pub const KP_MEMMULTIPLY: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMMULTIPLY as i32);
        pub const KP_MEMDIVIDE: Keycode = Keycode(SDL_KeyCode::SDLK_KP_MEMDIVIDE as i32);
        pub const KP_PLUSMINUS: Keycode = Keycode(SDL_KeyCode::SDLK_KP_PLUSMINUS as i32);
        pub const KP_CLEAR: Keycode = Keycode(SDL_KeyCode::SDLK_KP_CLEAR as i32);
        pub const KP_CLEARENTRY: Keycode = Keycode(SDL_KeyCode::SDLK_KP_CLEARENTRY as i32);
        pub const KP_BINARY: Keycode = Keycode(SDL_KeyCode::SDLK_KP_BINARY as i32);
        pub const KP_OCTAL: Keycode = Keycode(SDL_KeyCode::SDLK_KP_OCTAL as i32);
        pub const KP_DECIMAL: Keycode = Keycode(SDL_KeyCode::SDLK_KP_DECIMAL as i32);
        pub const KP_HEXADECIMAL: Keycode = Keycode(SDL_KeyCode::SDLK_KP_HEXADECIMAL as i32);
        pub const LCTRL: Keycode = Keycode(SDL_KeyCode::SDLK_LCTRL as i32);
        pub const LSHIFT: Keycode = Keycode(SDL_KeyCode::SDLK_LSHIFT as i32);
        pub const LALT: Keycode = Keycode(SDL_KeyCode::SDLK_LALT as i32);
        pub const LGUI: Keycode = Keycode(SDL_KeyCode::SDLK_LGUI as i32);
        pub const RCTRL: Keycode = Keycode(SDL_KeyCode::SDLK_RCTRL as i32);
        pub const RSHIFT: Keycode = Keycode(SDL_KeyCode::SDLK_RSHIFT as i32);
        pub const RALT: Keycode = Keycode(SDL_KeyCode::SDLK_RALT as i32);
        pub const RGUI: Keycode = Keycode(SDL_KeyCode::SDLK_RGUI as i32);
        pub const MODE: Keycode = Keycode(SDL_KeyCode::SDLK_MODE as i32);
        pub const AUDIONEXT: Keycode = Keycode(SDL_KeyCode::SDLK_AUDIONEXT as i32);
        pub const AUDIOPREV: Keycode = Keycode(SDL_KeyCode::SDLK_AUDIOPREV as i32);
        pub const AUDIOSTOP: Keycode = Keycode(SDL_KeyCode::SDLK_AUDIOSTOP as i32);
        pub const AUDIOPLAY: Keycode = Keycode(SDL_KeyCode::SDLK_AUDIOPLAY as i32);
        pub const AUDIOMUTE: Keycode = Keycode(SDL_KeyCode::SDLK_AUDIOMUTE as i32);
        pub const MEDIASELECT: Keycode = Keycode(SDL_KeyCode::SDLK_MEDIASELECT as i32);
        pub const WWW: Keycode = Keycode(SDL_KeyCode::SDLK_WWW as i32);
        pub const MAIL: Keycode = Keycode(SDL_KeyCode::SDLK_MAIL as i32);
        pub const CALCULATOR: Keycode = Keycode(SDL_KeyCode::SDLK_CALCULATOR as i32);
        pub const COMPUTER: Keycode = Keycode(SDL_KeyCode::SDLK_COMPUTER as i32);
        pub const AC_SEARCH: Keycode = Keycode(SDL_KeyCode::SDLK_AC_SEARCH as i32);
        pub const AC_HOME: Keycode = Keycode(SDL_KeyCode::SDLK_AC_HOME as i32);
        pub const AC_BACK: Keycode = Keycode(SDL_KeyCode::SDLK_AC_BACK as i32);
        pub const AC_FORWARD: Keycode = Keycode(SDL_KeyCode::SDLK_AC_FORWARD as i32);
        pub const AC_STOP: Keycode = Keycode(SDL_KeyCode::SDLK_AC_STOP as i32);
        pub const AC_REFRESH: Keycode = Keycode(SDL_KeyCode::SDLK_AC_REFRESH as i32);
        pub const AC_BOOKMARKS: Keycode = Keycode(SDL_KeyCode::SDLK_AC_BOOKMARKS as i32);
        pub const BRIGHTNESSDOWN: Keycode = Keycode(SDL_KeyCode::SDLK_BRIGHTNESSDOWN as i32);
        pub const BRIGHTNESSUP: Keycode = Keycode(SDL_KeyCode::SDLK_BRIGHTNESSUP as i32);
        pub const DISPLAYSWITCH: Keycode = Keycode(SDL_KeyCode::SDLK_DISPLAYSWITCH as i32);
        pub const KBDILLUMTOGGLE: Keycode = Keycode(SDL_KeyCode::SDLK_KBDILLUMTOGGLE as i32);
        pub const KBDILLUMDOWN: Keycode = Keycode(SDL_KeyCode::SDLK_KBDILLUMDOWN as i32);
        pub const KBDILLUMUP: Keycode = Keycode(SDL_KeyCode::SDLK_KBDILLUMUP as i32);
        pub const EJECT: Keycode = Keycode(SDL_KeyCode::SDLK_EJECT as i32);
        pub const SLEEP: Keycode = Keycode(SDL_KeyCode::SDLK_SLEEP as i32);
    }

    #[repr(u32)]
    #[derive(Copy, Clone, Hash, PartialEq, Eq)]
    enum SDL_KeyCode {
        SDLK_UNKNOWN = 0,
        SDLK_RETURN = 13,
        SDLK_ESCAPE = 27,
        SDLK_BACKSPACE = 8,
        SDLK_TAB = 9,
        SDLK_SPACE = 32,
        SDLK_EXCLAIM = 33,
        SDLK_QUOTEDBL = 34,
        SDLK_HASH = 35,
        SDLK_PERCENT = 37,
        SDLK_DOLLAR = 36,
        SDLK_AMPERSAND = 38,
        SDLK_QUOTE = 39,
        SDLK_LEFTPAREN = 40,
        SDLK_RIGHTPAREN = 41,
        SDLK_ASTERISK = 42,
        SDLK_PLUS = 43,
        SDLK_COMMA = 44,
        SDLK_MINUS = 45,
        SDLK_PERIOD = 46,
        SDLK_SLASH = 47,
        SDLK_0 = 48,
        SDLK_1 = 49,
        SDLK_2 = 50,
        SDLK_3 = 51,
        SDLK_4 = 52,
        SDLK_5 = 53,
        SDLK_6 = 54,
        SDLK_7 = 55,
        SDLK_8 = 56,
        SDLK_9 = 57,
        SDLK_COLON = 58,
        SDLK_SEMICOLON = 59,
        SDLK_LESS = 60,
        SDLK_EQUALS = 61,
        SDLK_GREATER = 62,
        SDLK_QUESTION = 63,
        SDLK_AT = 64,
        SDLK_LEFTBRACKET = 91,
        SDLK_BACKSLASH = 92,
        SDLK_RIGHTBRACKET = 93,
        SDLK_CARET = 94,
        SDLK_UNDERSCORE = 95,
        SDLK_BACKQUOTE = 96,
        SDLK_a = 97,
        SDLK_b = 98,
        SDLK_c = 99,
        SDLK_d = 100,
        SDLK_e = 101,
        SDLK_f = 102,
        SDLK_g = 103,
        SDLK_h = 104,
        SDLK_i = 105,
        SDLK_j = 106,
        SDLK_k = 107,
        SDLK_l = 108,
        SDLK_m = 109,
        SDLK_n = 110,
        SDLK_o = 111,
        SDLK_p = 112,
        SDLK_q = 113,
        SDLK_r = 114,
        SDLK_s = 115,
        SDLK_t = 116,
        SDLK_u = 117,
        SDLK_v = 118,
        SDLK_w = 119,
        SDLK_x = 120,
        SDLK_y = 121,
        SDLK_z = 122,
        SDLK_CAPSLOCK = 1073741881,
        SDLK_F1 = 1073741882,
        SDLK_F2 = 1073741883,
        SDLK_F3 = 1073741884,
        SDLK_F4 = 1073741885,
        SDLK_F5 = 1073741886,
        SDLK_F6 = 1073741887,
        SDLK_F7 = 1073741888,
        SDLK_F8 = 1073741889,
        SDLK_F9 = 1073741890,
        SDLK_F10 = 1073741891,
        SDLK_F11 = 1073741892,
        SDLK_F12 = 1073741893,
        SDLK_PRINTSCREEN = 1073741894,
        SDLK_SCROLLLOCK = 1073741895,
        SDLK_PAUSE = 1073741896,
        SDLK_INSERT = 1073741897,
        SDLK_HOME = 1073741898,
        SDLK_PAGEUP = 1073741899,
        SDLK_DELETE = 127,
        SDLK_END = 1073741901,
        SDLK_PAGEDOWN = 1073741902,
        SDLK_RIGHT = 1073741903,
        SDLK_LEFT = 1073741904,
        SDLK_DOWN = 1073741905,
        SDLK_UP = 1073741906,
        SDLK_NUMLOCKCLEAR = 1073741907,
        SDLK_KP_DIVIDE = 1073741908,
        SDLK_KP_MULTIPLY = 1073741909,
        SDLK_KP_MINUS = 1073741910,
        SDLK_KP_PLUS = 1073741911,
        SDLK_KP_ENTER = 1073741912,
        SDLK_KP_1 = 1073741913,
        SDLK_KP_2 = 1073741914,
        SDLK_KP_3 = 1073741915,
        SDLK_KP_4 = 1073741916,
        SDLK_KP_5 = 1073741917,
        SDLK_KP_6 = 1073741918,
        SDLK_KP_7 = 1073741919,
        SDLK_KP_8 = 1073741920,
        SDLK_KP_9 = 1073741921,
        SDLK_KP_0 = 1073741922,
        SDLK_KP_PERIOD = 1073741923,
        SDLK_APPLICATION = 1073741925,
        SDLK_POWER = 1073741926,
        SDLK_KP_EQUALS = 1073741927,
        SDLK_F13 = 1073741928,
        SDLK_F14 = 1073741929,
        SDLK_F15 = 1073741930,
        SDLK_F16 = 1073741931,
        SDLK_F17 = 1073741932,
        SDLK_F18 = 1073741933,
        SDLK_F19 = 1073741934,
        SDLK_F20 = 1073741935,
        SDLK_F21 = 1073741936,
        SDLK_F22 = 1073741937,
        SDLK_F23 = 1073741938,
        SDLK_F24 = 1073741939,
        SDLK_EXECUTE = 1073741940,
        SDLK_HELP = 1073741941,
        SDLK_MENU = 1073741942,
        SDLK_SELECT = 1073741943,
        SDLK_STOP = 1073741944,
        SDLK_AGAIN = 1073741945,
        SDLK_UNDO = 1073741946,
        SDLK_CUT = 1073741947,
        SDLK_COPY = 1073741948,
        SDLK_PASTE = 1073741949,
        SDLK_FIND = 1073741950,
        SDLK_MUTE = 1073741951,
        SDLK_VOLUMEUP = 1073741952,
        SDLK_VOLUMEDOWN = 1073741953,
        SDLK_KP_COMMA = 1073741957,
        SDLK_KP_EQUALSAS400 = 1073741958,
        SDLK_ALTERASE = 1073741977,
        SDLK_SYSREQ = 1073741978,
        SDLK_CANCEL = 1073741979,
        SDLK_CLEAR = 1073741980,
        SDLK_PRIOR = 1073741981,
        SDLK_RETURN2 = 1073741982,
        SDLK_SEPARATOR = 1073741983,
        SDLK_OUT = 1073741984,
        SDLK_OPER = 1073741985,
        SDLK_CLEARAGAIN = 1073741986,
        SDLK_CRSEL = 1073741987,
        SDLK_EXSEL = 1073741988,
        SDLK_KP_00 = 1073742000,
        SDLK_KP_000 = 1073742001,
        SDLK_THOUSANDSSEPARATOR = 1073742002,
        SDLK_DECIMALSEPARATOR = 1073742003,
        SDLK_CURRENCYUNIT = 1073742004,
        SDLK_CURRENCYSUBUNIT = 1073742005,
        SDLK_KP_LEFTPAREN = 1073742006,
        SDLK_KP_RIGHTPAREN = 1073742007,
        SDLK_KP_LEFTBRACE = 1073742008,
        SDLK_KP_RIGHTBRACE = 1073742009,
        SDLK_KP_TAB = 1073742010,
        SDLK_KP_BACKSPACE = 1073742011,
        SDLK_KP_A = 1073742012,
        SDLK_KP_B = 1073742013,
        SDLK_KP_C = 1073742014,
        SDLK_KP_D = 1073742015,
        SDLK_KP_E = 1073742016,
        SDLK_KP_F = 1073742017,
        SDLK_KP_XOR = 1073742018,
        SDLK_KP_POWER = 1073742019,
        SDLK_KP_PERCENT = 1073742020,
        SDLK_KP_LESS = 1073742021,
        SDLK_KP_GREATER = 1073742022,
        SDLK_KP_AMPERSAND = 1073742023,
        SDLK_KP_DBLAMPERSAND = 1073742024,
        SDLK_KP_VERTICALBAR = 1073742025,
        SDLK_KP_DBLVERTICALBAR = 1073742026,
        SDLK_KP_COLON = 1073742027,
        SDLK_KP_HASH = 1073742028,
        SDLK_KP_SPACE = 1073742029,
        SDLK_KP_AT = 1073742030,
        SDLK_KP_EXCLAM = 1073742031,
        SDLK_KP_MEMSTORE = 1073742032,
        SDLK_KP_MEMRECALL = 1073742033,
        SDLK_KP_MEMCLEAR = 1073742034,
        SDLK_KP_MEMADD = 1073742035,
        SDLK_KP_MEMSUBTRACT = 1073742036,
        SDLK_KP_MEMMULTIPLY = 1073742037,
        SDLK_KP_MEMDIVIDE = 1073742038,
        SDLK_KP_PLUSMINUS = 1073742039,
        SDLK_KP_CLEAR = 1073742040,
        SDLK_KP_CLEARENTRY = 1073742041,
        SDLK_KP_BINARY = 1073742042,
        SDLK_KP_OCTAL = 1073742043,
        SDLK_KP_DECIMAL = 1073742044,
        SDLK_KP_HEXADECIMAL = 1073742045,
        SDLK_LCTRL = 1073742048,
        SDLK_LSHIFT = 1073742049,
        SDLK_LALT = 1073742050,
        SDLK_LGUI = 1073742051,
        SDLK_RCTRL = 1073742052,
        SDLK_RSHIFT = 1073742053,
        SDLK_RALT = 1073742054,
        SDLK_RGUI = 1073742055,
        SDLK_MODE = 1073742081,
        SDLK_AUDIONEXT = 1073742082,
        SDLK_AUDIOPREV = 1073742083,
        SDLK_AUDIOSTOP = 1073742084,
        SDLK_AUDIOPLAY = 1073742085,
        SDLK_AUDIOMUTE = 1073742086,
        SDLK_MEDIASELECT = 1073742087,
        SDLK_WWW = 1073742088,
        SDLK_MAIL = 1073742089,
        SDLK_CALCULATOR = 1073742090,
        SDLK_COMPUTER = 1073742091,
        SDLK_AC_SEARCH = 1073742092,
        SDLK_AC_HOME = 1073742093,
        SDLK_AC_BACK = 1073742094,
        SDLK_AC_FORWARD = 1073742095,
        SDLK_AC_STOP = 1073742096,
        SDLK_AC_REFRESH = 1073742097,
        SDLK_AC_BOOKMARKS = 1073742098,
        SDLK_BRIGHTNESSDOWN = 1073742099,
        SDLK_BRIGHTNESSUP = 1073742100,
        SDLK_DISPLAYSWITCH = 1073742101,
        SDLK_KBDILLUMTOGGLE = 1073742102,
        SDLK_KBDILLUMDOWN = 1073742103,
        SDLK_KBDILLUMUP = 1073742104,
        SDLK_EJECT = 1073742105,
        SDLK_SLEEP = 1073742106,
        SDLK_APP1 = 1073742107,
        SDLK_APP2 = 1073742108,
        SDLK_AUDIOREWIND = 1073742109,
        SDLK_AUDIOFASTFORWARD = 1073742110,
        SDLK_SOFTLEFT = 1073742111,
        SDLK_SOFTRIGHT = 1073742112,
        SDLK_CALL = 1073742113,
        SDLK_ENDCALL = 1073742114,
    }

    bitflags::bitflags! {
        pub struct Mod: u16 {
            const NOMOD = 0x0000;
            const LSHIFTMOD = 0x0001;
            const RSHIFTMOD = 0x0002;
            const LCTRLMOD = 0x0040;
            const RCTRLMOD = 0x0080;
            const LALTMOD = 0x0100;
            const RALTMOD = 0x0200;
            const LGUIMOD = 0x0400;
            const RGUIMOD = 0x0800;
            const NUMMOD = 0x1000;
            const CAPSMOD = 0x2000;
            const MODEMOD = 0x4000;
            const RESERVEDMOD = 0x8000;
        }
    }
}
#[cfg(feature = "sdl2")]
mod enums {
    pub use sdl2::keyboard::{Keycode, Mod, Scancode};
}
