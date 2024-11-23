use windows::Win32::Foundation::COLORREF;

/// Emulates the GDI `RGB()` macro.
pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(((r as u32) << 0) | ((g as u32) << 8) | ((b as u32) << 16))
}