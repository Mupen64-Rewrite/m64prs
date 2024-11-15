use std::{
    env, fs,
    path::PathBuf,
};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn copy_stuff_to_target() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap();

    let native_target_dir = PathBuf::from(m64prs_native::NATIVE_TARGET_DIR);

    for entry in fs::read_dir(native_target_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
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
    copy_stuff_to_target();
    gl_gen();
}
