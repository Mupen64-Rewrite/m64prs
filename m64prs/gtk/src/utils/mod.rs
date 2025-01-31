pub mod dpi_conv;
pub mod keyboard;
pub mod paths;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl.gen.rs"));
}
