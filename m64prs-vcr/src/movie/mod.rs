use helpers::fix_buttons_order;
use m64prs_sys::Buttons;
use std::{
    ffi::c_int, fmt::Debug, io::{self, Read, Write}, mem::{self}
};

pub mod error;
mod helpers;

pub use helpers::{StringField, AsciiField};

pub const M64_MAGIC: [u8; 4] = [b'M', b'6', b'4', 0x1Au8];

#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct M64Header {
    /// The signature of the .m64 file.
    magic: [u8; 4],
    version: u32,
    /// A unique UID associated with the movie. Generally equal to the Unix timestamp of its creation,
    /// though that will overflow in 2106.
    pub uid: u32,
    /// Time needed to play the movie, in VIs (vertical interrupts).
    pub length_vis: u32,
    /// Number of rerecords for this movie.
    pub rerecord_count: u32,
    /// Framerate of the console. Generally 60 on NTSC consoles and 50 on PAL.
    pub vis_per_second: u8,
    /// Number of controllers connected for this.
    pub num_controllers: u8,
    _reserved1: u16,
    /// Number of input frames in the movie.
    pub length_samples: u32,
    /// How the movie should be started.
    pub start_flags: StartType,
    _reserved2: u16,
    /// Flags specifying which controllers and which attachments are connected.
    pub controller_flags: ControllerFlags,
    _reserved3: [u8; 160],
    /// Internal name of the ROM used to record this movie.
    pub rom_name: AsciiField<32>,
    /// ROM's CRC32, ripped from its header.
    pub rom_crc: u32,
    /// ROM's country code, ripped from its header.
    pub rom_cc: u16,
    _reserved4: [u8; 56],
    /// Internal name of the video plugin used to record this movie.
    pub graphics_plugin: AsciiField<64>,
    /// Internal name of the audio plugin used to record this movie.
    pub audio_plugin: AsciiField<64>,
    /// Internal name of the input plugin used to record this movie.
    pub input_plugin: AsciiField<64>,
    /// Internal name of the RSP plugin used to record this movie.
    pub rsp_plugin: AsciiField<64>,
    /// Info on the movie's authors.
    pub author: StringField<222>,
    /// Description of the movie.
    pub description: StringField<256>,
}

// Check that struct conforms to spec (even with alignment)
const _: () = {
    use std::mem;
    assert!(mem::size_of::<M64Header>() == 1024);
    assert!(mem::offset_of!(M64Header, magic) == 0x000);
    assert!(mem::offset_of!(M64Header, version) == 0x004);
    assert!(mem::offset_of!(M64Header, uid) == 0x008);
    assert!(mem::offset_of!(M64Header, length_vis) == 0x00C);
    assert!(mem::offset_of!(M64Header, rerecord_count) == 0x010);
    assert!(mem::offset_of!(M64Header, vis_per_second) == 0x014);
    assert!(mem::offset_of!(M64Header, num_controllers) == 0x015);
    assert!(mem::offset_of!(M64Header, length_samples) == 0x018);
    assert!(mem::offset_of!(M64Header, start_flags) == 0x01C);
    assert!(mem::offset_of!(M64Header, controller_flags) == 0x020);
    assert!(mem::offset_of!(M64Header, rom_name) == 0x0C4);
    assert!(mem::offset_of!(M64Header, rom_crc) == 0x0E4);
    assert!(mem::offset_of!(M64Header, rom_cc) == 0x0E8);
    assert!(mem::offset_of!(M64Header, graphics_plugin) == 0x122);
    assert!(mem::offset_of!(M64Header, audio_plugin) == 0x162);
    assert!(mem::offset_of!(M64Header, input_plugin) == 0x1A2);
    assert!(mem::offset_of!(M64Header, rsp_plugin) == 0x1E2);
    assert!(mem::offset_of!(M64Header, author) == 0x222);
    assert!(mem::offset_of!(M64Header, description) == 0x300);
};

impl Default for M64Header {
    fn default() -> Self {
        Self {
            magic: M64_MAGIC,
            version: 3,
            uid: 0,
            length_vis: u32::MAX,
            rerecord_count: u32::MAX,
            vis_per_second: 60,
            num_controllers: 1,
            _reserved1: 0,
            length_samples: 0,
            start_flags: StartType::FROM_SNAPSHOT,
            _reserved2: 0,
            controller_flags: ControllerFlags::P1_PRESENT,
            _reserved3: [0; 160],
            rom_name: Default::default(),
            rom_crc: 0,
            rom_cc: 0,
            _reserved4: [0; 56],
            graphics_plugin: Default::default(),
            audio_plugin: Default::default(),
            input_plugin: Default::default(),
            rsp_plugin: Default::default(),
            author: Default::default(),
            description: Default::default(),
        }
    }
}

impl M64Header {
    fn from_bytes(slice: [u8; 1024]) -> Self {
        // SAFETY: All fields, including padding, do not technically have invalid values.
        let mut result = unsafe { mem::transmute::<[u8; 1024], Self>(slice) };

        // Fix endianness of integer fields
        macro_rules! fix_field {
            ($name:ident) => {
                result.$name = result.$name.to_le()
            };
            (newtype $name:ident) => {
                result.$name.0 = result.$name.0.to_le()
            };
            (bitflag $name:ident) => {
                result.$name.0 .0 = result.$name.0 .0.to_le()
            };
        }

        fix_field!(version);
        fix_field!(uid);
        fix_field!(length_vis);
        fix_field!(rerecord_count);
        fix_field!(length_samples);
        fix_field!(newtype start_flags);
        fix_field!(bitflag controller_flags);
        fix_field!(rom_crc);
        fix_field!(rom_cc);

        result
    }

    fn into_bytes(mut self) -> [u8; 1024] {
        // Un-fix endianness of integer fields.
        macro_rules! fix_field {
            ($name:ident) => {
                self.$name = self.$name.to_le()
            };
            (newtype $name:ident) => {
                self.$name.0 = self.$name.0.to_le()
            };
            (bitflag $name:ident) => {
                self.$name.0 .0 = self.$name.0 .0.to_le()
            };
        }

        fix_field!(version);
        fix_field!(uid);
        fix_field!(length_vis);
        fix_field!(rerecord_count);
        fix_field!(length_samples);
        fix_field!(newtype start_flags);
        fix_field!(bitflag controller_flags);
        fix_field!(rom_crc);
        fix_field!(rom_cc);

        // SAFETY: M64Header is a POD type and can validly be converted
        // to and from its byte representation.
        unsafe { mem::transmute(self) }
    }
}

impl Debug for M64Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("M64Header")
            .field("magic", &self.magic)
            .field("version", &self.version)
            .field("uid", &self.uid)
            .field("length_vis", &self.length_vis)
            .field("rerecord_count", &self.rerecord_count)
            .field("vis_per_second", &self.vis_per_second)
            .field("num_controllers", &self.num_controllers)
            .field("length_samples", &self.length_samples)
            .field("start_flags", &self.start_flags)
            .field("controller_flags", &self.controller_flags)
            .field("rom_name", &self.rom_name)
            .field("rom_crc", &self.rom_crc)
            .field("rom_cc", &self.rom_cc)
            .field("video_plugin", &self.graphics_plugin)
            .field("audio_plugin", &self.audio_plugin)
            .field("input_plugin", &self.input_plugin)
            .field("rsp_plugin", &self.rsp_plugin)
            .field("author", &self.author)
            .field("description", &self.description)
            .finish()
    }
}

/// Value indicating how the .m64 file is to be started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartType(pub u16);

impl StartType {
    /// Indicates that a savestate associated with the .m64 file should be loaded.
    pub const FROM_SNAPSHOT: StartType = StartType(1 << 0);
    /// Indicates that the game should be reset.
    pub const FROM_RESET: StartType = StartType(1 << 1);
    /// Indicates that an EEPROM dump associated with the .m64 file
    pub const FROM_EEPROM: StartType = StartType(1 << 2);
}

bitflags::bitflags! {
    /// Flags indicating which controllers are present and what attachments they have.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ControllerFlags: u32 {
        const NONE = 0;

        /// Port 1 has a controller connected.
        const P1_PRESENT = 1 << 0;
        /// Port 2 has a controller connected.
        const P2_PRESENT = 1 << 1;
        /// Port 3 has a controller connected.
        const P3_PRESENT = 1 << 2;
        /// Port 4 has a controller connected.
        const P4_PRESENT = 1 << 3;

        /// Port 1 has a Memory Pak attached to the controller.
        const P1_MEM_PAK = 1 << 4;
        /// Port 2 has a Memory Pak attached to the controller.
        const P2_MEM_PAK = 1 << 5;
        /// Port 3 has a Memory Pak attached to the controller.
        const P3_MEM_PAK = 1 << 6;
        /// Port 4 has a Memory Pak attached to the controller.
        const P4_MEM_PAK = 1 << 7;

        /// Port 1 has a Rumble Pak attached to the controller.
        const P1_RUMBLE_PAK = 1 << 8;
        /// Port 2 has a Rumble Pak attached to the controller.
        const P2_RUMBLE_PAK = 1 << 9;
        /// Port 3 has a Rumble Pak attached to the controller.
        const P3_RUMBLE_PAK = 1 << 10;
        /// Port 4 has a Rumble Pak attached to the controller.
        const P4_RUMBLE_PAK = 1 << 11;
    }
}

impl ControllerFlags {
    pub fn port_present(self, port: c_int) -> bool {
        assert!((0..4).contains(&port));
        
        self.contains(Self::from_bits_retain(1 << port))
    }
    pub fn port_has_mempak(self, port: c_int) -> bool {
        assert!((0..4).contains(&port));
        
        self.contains(Self::from_bits_retain((1 << 4) << port))
    }
    pub fn port_has_rumblepak(self, port: c_int) -> bool {
        assert!((0..4).contains(&port));
        
        self.contains(Self::from_bits_retain((1 << 8) << port))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct M64File {
    pub header: M64Header,
    pub inputs: Vec<Buttons>,
}

impl M64File {
    pub fn read_from<R: Read>(mut reader: R) -> io::Result<Self> {
        let header = {
            // Try to read the header (exactly 1024 bytes)
            let mut buffer = [0u8; mem::size_of::<M64Header>()];
            reader.read_exact(&mut buffer)?;
            // Check signature
            if buffer[0..4] != M64_MAGIC {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(".m64: signature doesn't match ({:x?})", &buffer[0..4]),
                ));
            }
            // Parse the header
            M64Header::from_bytes(buffer)
        };
        let inputs = {
            // Compute buffer sizes
            let buffer_size: usize = header.length_samples.try_into().unwrap();
            let buffer_byte_size: usize =
                buffer_size.checked_mul(mem::size_of::<Buttons>()).unwrap();

            // Read remaining bytes, it should all be input data
            let input_bytes = {
                let mut buffer = Vec::<u8>::new();
                reader.read_to_end(&mut buffer)?;
                buffer.into_boxed_slice()
            };

            // Ensure we have as many samples as the header says we do
            if input_bytes.len() < buffer_byte_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    ".m64: not enough input frames",
                ));
            }
            // Setup the input vector. Ideally it should not need to be zeroed.
            let mut buffer = vec![Buttons::BLANK; buffer_size];

            // Copy the input bytes directly into the buttons and fix any endianness issues.
            unsafe {
                // SAFETY: Buttons has no invalid values.
                let bytes = buffer.align_to_mut::<u8>().1;
                bytes.copy_from_slice(&input_bytes[0..buffer_byte_size]);
            }
            fix_buttons_order(&mut buffer);

            buffer
        };

        Ok(Self { header, inputs })
    }

    pub fn write_into<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let Self { header, mut inputs } = self;
        writer.write_all(&header.into_bytes())?;

        fix_buttons_order(&mut inputs);
        unsafe {
            // SAFETY: Buttons is a POD type and can be directly written.
            let bytes = inputs.align_to_mut::<u8>().1;
            writer.write_all(bytes)?;
        }

        Ok(())
    }
}
