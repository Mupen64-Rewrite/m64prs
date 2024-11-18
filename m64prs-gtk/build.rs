use std::{env, fs, path::PathBuf};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

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
    // copy_stuff_to_target();
    gl_gen();
}
