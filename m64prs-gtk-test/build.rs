use std::{
    env, fs,
    path::PathBuf,
};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn copy_stuff_to_target() {
    const EXT_BLACKLIST: [&str; 3] = ["ilk", "lib", "exp"];

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap();

    let native_target_dir = PathBuf::from(m64prs_native::NATIVE_TARGET_DIR);

    for entry in fs::read_dir(native_target_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            let file_name = entry.file_name().into_string().unwrap();
            let file_ext = file_name.split_once('.').map(|(_, ext)| ext);

            // remove blacklisted files (MSVC link databases, import libraries, export definitions)
            if file_ext.is_some_and(|file_ext| EXT_BLACKLIST.iter().any(|ext| file_ext.starts_with(ext))) {
                continue;
            }
            fs::copy(&path, target_dir.join(file_name)).unwrap();
        }
    }
}

fn gl_gen() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_path = out_dir.join("gl.gen.rs");

    Registry::new(Api::Gl, (3, 3), Profile::Compatibility, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut fs::File::create(out_path).unwrap())
        .expect("gl_generator failed");
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../m64prs-native/target");
    copy_stuff_to_target();
    gl_gen();
}
