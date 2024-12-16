use m64prs_sys::ButtonFlags;


#[glib::flags(name = "TasDiGButtonFlags")]
pub enum GButtonFlags {
    #[flags_value(skip)]
    NONE = 0,
    #[flags_value(name = "DR")]
    D_RIGHT = 1 << 0,
    #[flags_value(name = "DL")]
    D_LEFT = 1 << 1,
    #[flags_value(name = "DD")]
    D_DOWN = 1 << 2,
    #[flags_value(name = "DU")]
    D_UP = 1 << 3,

    #[flags_value(name = "Start")]
    START = 1 << 4,
    #[flags_value(name = "Z")]
    Z = 1 << 5,
    #[flags_value(name = "B")]
    B = 1 << 6,
    #[flags_value(name = "A")]
    A = 1 << 7,

    #[flags_value(name = "CR")]
    C_RIGHT = 1 << 8,
    #[flags_value(name = "CL")]
    C_LEFT = 1 << 9,
    #[flags_value(name = "CD")]
    C_DOWN = 1 << 10,
    #[flags_value(name = "CU")]
    C_UP = 1 << 11,

    #[flags_value(name = "R")]
    R = 1 << 12,
    #[flags_value(name = "L")]
    L = 1 << 13,

    #[flags_value(name = "Reserved 1")]
    RESERVED1 = 1 << 14,
    #[flags_value(name = "Reserved 2")]
    RESERVED2 = 1 << 15,
}

impl Default for GButtonFlags {
    fn default() -> Self {
        Self::NONE
    }
}

impl From<ButtonFlags> for GButtonFlags {
    fn from(value: ButtonFlags) -> Self {
        Self::from_bits_retain(value.bits() as u32)
    }
}

impl From<GButtonFlags> for ButtonFlags {
    fn from(value: GButtonFlags) -> Self {
        Self::from_bits_retain(value.bits() as u16)
    }
}