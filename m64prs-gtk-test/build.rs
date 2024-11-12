use std::{
    env, fs,
    path::PathBuf,
};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    
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
