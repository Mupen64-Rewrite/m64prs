use std::{borrow::Borrow, env, fs, path::PathBuf};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/build/dirs.rs"));

pub const WIN32_DEPS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    path_sep!(),
    "mupen64plus-win32-deps"
);

pub fn link_sdl_win32() {
    let win32_deps_dir = PathBuf::from(WIN32_DEPS_DIR);

    let sdl2_base_path = fs::read_dir(&win32_deps_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name().to_string_lossy().starts_with("SDL2-")).then(|| entry.path())
        })
        .next()
        .expect("mupen64plus-win32-deps should have SDL2");

    let sdl2_lib_path = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().borrow() {
        "x86_64" => sdl2_base_path.join("lib\\x64"),
        "x86" => sdl2_base_path.join("lib\\x86"),
        _ => panic!("Architecture unsupported!"),
    };

    println!(
        "cargo::rustc-link-search=native={}",
        &sdl2_lib_path.to_string_lossy()
    );
}
