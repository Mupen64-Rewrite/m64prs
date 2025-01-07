pub mod dirs;
pub mod dpi_conv;
pub mod keyboard;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl.gen.rs"));
}
