use std::{
    env, fs,
    path::{Path, PathBuf},
};

mod dirs;
#[cfg(unix)]
mod make;
#[cfg(windows)]
mod msvc;

pub fn setup_cargo_reruns() {
    fn emit(path: &Path) {
        println!("cargo::rerun-if-changed={}", path.to_str().unwrap())
    }

    println!("cargo::rerun-if-changed=build.rs");
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
    let (vs_env_arch, msbuild_platform) = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str()
    {
        "x86" => ("x86", "Win32"),
        "x86_64" => ("amd64", "x64"),
        _ => panic!("Target platform not supported!"),
    };
    let msbuild_config = match env::var("PROFILE").unwrap().as_str() {
        "debug" => "Debug",
        "release" => "Release",
        _ => unreachable!(),
    };

    let vs_env = msvc::vs_dev_env(vs_env_arch);

    let root_path = Path::new(dirs::ROOT_DIR);
    let sln_file = root_path.join("m64prs-vs-deps.sln");

    msvc::msbuild(
        &vs_env,
        &sln_file,
        &out_dir,
        &msbuild_config,
        &msbuild_platform,
    );
}

#[cfg(unix)]
fn compile_m64p_deps(out_dir: &Path) {
    use std::fs;

    let core_dir = PathBuf::from(dirs::M64P_CORE_DIR);
    let makefile_dir = core_dir.join("projects/unix");

    make::make(&makefile_dir, ["all", "TAS=1"]);
    for file in fs::read_dir(makefile_dir).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        let name = file.file_name().into_string().unwrap();
        if let Some((name, extension)) = name.split_once('.') {
            if !(extension.starts_with("so") || extension.starts_with("dylib")) {
                continue;
            }
            let strip_ext = name
                .split_once('.')
                .map(|(ext1, _)| ext1)
                .unwrap_or(name);

            let out_path = out_dir.join(format!("{}.{}", name, strip_ext));
            fs::copy(&path, &out_path).expect("copy to target dir should succeed");
        }
    }
}

fn main() {
    // ./target contains all native outputs
    let out_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("target");
    fs::create_dir_all(&out_dir).unwrap();

    setup_cargo_reruns();
    compile_m64p_deps(&out_dir);
}
