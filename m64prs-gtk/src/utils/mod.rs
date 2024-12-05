pub mod actions;
pub mod dpi_conv;
pub mod t_option;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl.gen.rs"));
}
