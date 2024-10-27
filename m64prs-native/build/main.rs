use std::{env, path::{Path, PathBuf}};

mod dirs;
#[cfg(windows)]
mod msvc;

pub fn setup_cargo_reruns() {
    fn emit(path: &Path) {
        println!("cargo::rerun-if-changed={}", path.to_str().unwrap())
    }

    // mupen64plus-core-tas
    {
        let core_dir = Path::new(dirs::M64P_CORE_DIR);
        emit(&core_dir.join("src"));
        #[cfg(windows)]
        emit(&core_dir.join("projects/msvc"));
        #[cfg(unix)]
        emit(&core_dir.join("projects/unix"));
    }
}


#[cfg(windows)]
fn compile_m64p_deps(out_dir: &Path) {
    let (vs_env_arch, msbuild_platform) = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "x86" => ("x86", "Win32"),
        "x86_64" => ("amd64", "x64"),
        _ => panic!("Target platform not supported!")
    };
    let msbuild_config = match env::var("PROFILE").unwrap().as_str() {
        "debug" => "Debug",
        "release" => "Release",
        _ => unreachable!()
    };

    let vs_env = msvc::vs_dev_env(vs_env_arch);

    let sln_file = Path::new(dirs::ROOT_DIR).join("m64prs-vs-deps.sln");

    msvc::msbuild(&vs_env, &sln_file, &out_dir, &msbuild_config, &msbuild_platform);
}

#[cfg(unix)]
fn compile_m64p_deps(out_dir: &Path) {

}

fn main() {
    let out_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("target");

    compile_m64p_deps(&out_dir);
}