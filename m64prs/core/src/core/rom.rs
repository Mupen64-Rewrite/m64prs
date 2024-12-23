use std::{
    ffi::{c_int, c_void},
    mem,
};

use m64prs_sys::{common::M64PError, Command, RomHeader, RomSettings};

use super::Core;

impl Core {
    /// Opens a ROM that is pre-loaded into memory.
    pub fn open_rom(&mut self, rom_data: &[u8]) -> Result<(), M64PError> {
        unsafe {
            // SAFETY: Mupen64Plus copies the ROM data passed into this function.
            // This means that it won't be invalidated if the ROM data borrowed here
            // goes out of scope.
            self.do_command_ip(
                Command::RomOpen,
                rom_data
                    .len()
                    .try_into()
                    .expect("size of ROM should fit into c_int"),
                rom_data.as_ptr() as *mut c_void,
            )
        }
    }

    /// Closes a currently open ROM.
    pub fn close_rom(&mut self) -> Result<(), M64PError> {
        self.do_command(Command::RomClose)
    }

    /// Extracts the ROM header from the currently open ROM.
    pub fn rom_header(&self) -> Result<RomHeader, M64PError> {
        unsafe {
            let mut rom_header = RomHeader::default();

            self.do_command_ip(
                Command::RomGetHeader,
                mem::size_of::<RomHeader>() as c_int,
                &mut rom_header as *mut _ as *mut c_void,
            )?;

            Ok(rom_header)
        }
    }

    /// Extracts the ROM settings for the currently open ROM.
    pub fn rom_settings(&self) -> Result<RomSettings, M64PError> {
        unsafe {
            let mut rom_header = RomSettings::default();

            self.do_command_ip(
                Command::RomGetSettings,
                mem::size_of::<RomSettings>() as c_int,
                &mut rom_header as *mut _ as *mut c_void,
            )?;

            Ok(rom_header)
        }
    }
}
