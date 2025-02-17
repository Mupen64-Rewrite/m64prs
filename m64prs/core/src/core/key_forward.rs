use m64prs_sys::{
    key::{Mod, Scancode},
    Command,
};

use crate::error::M64PError;

use super::Core;

impl Core {
    pub fn forward_key_down(
        &self,
        sdl_key: Option<Scancode>,
        sdl_mod: Mod,
    ) -> Result<(), M64PError> {
        let sdl_key = (sdl_key.map_or(0, |key| key as i32) as u32) & 0xFFFFu32;
        let sdl_mod = sdl_mod.bits() as u32;

        let int_param = ((sdl_mod << 16) | sdl_key) as i32;

        self.do_command_i(Command::SendSdlKeydown, int_param)
    }
    pub fn forward_key_up(&self, sdl_key: Option<Scancode>, sdl_mod: Mod) -> Result<(), M64PError> {
        let sdl_key = (sdl_key.map_or(0, |key| key as i32) as u32) & 0xFFFFu32;
        let sdl_mod = sdl_mod.bits() as u32;

        let int_param = ((sdl_mod << 16) | sdl_key) as i32;

        self.do_command_i(Command::SendSdlKeyup, int_param)
    }
}
