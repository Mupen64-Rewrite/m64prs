#[cfg(unix)]
macro_rules! path_sep {
    () => {
        "/"
    };
}
#[cfg(windows)]
macro_rules! path_sep {
    () => {
        "\\"
    };
}

pub const M64P_CORE_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), path_sep!(), "mupen64plus-core");

pub const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");
