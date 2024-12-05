pub mod actions;
pub mod dpi_conv;
pub mod mp_lib;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl.gen.rs"));
}
