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

    // copy DLLs (w.r.t. target arch)
    let win32_libs = match msbuild_platform {
        "Win32" => vec![
            root_path.join("mupen64plus-win32-deps\\freetype-2.13.0\\lib\\x86\\freetype.dll"),
            root_path.join("mupen64plus-win32-deps\\SDL2_net-2.2.0\\lib\\x86\\SDL2_net.dll"),
            root_path.join("mupen64plus-win32-deps\\SDL2-2.26.3\\lib\\x86\\SDL2.dll"),
            root_path.join("mupen64plus-win32-deps\\zlib-1.2.13\\lib\\x86\\zlib.dll"),
        ],
        "x64" => vec![
            root_path.join("mupen64plus-win32-deps\\freetype-2.13.0\\lib\\x64\\freetype.dll"),
            root_path.join("mupen64plus-win32-deps\\SDL2_net-2.2.0\\lib\\x64\\SDL2_net.dll"),
            root_path.join("mupen64plus-win32-deps\\SDL2-2.26.3\\lib\\x64\\SDL2.dll"),
            root_path.join("mupen64plus-win32-deps\\zlib-1.2.13\\lib\\x64\\zlib.dll"),
        ],
        _ => unreachable!()
    };
    for lib_path in win32_libs {
        let file_name = lib_path.file_name().unwrap();
        fs::copy(&lib_path, out_dir.join(file_name)).unwrap();
    }

    for entry in fs::read_dir(root_path.join("mupen64plus-core-tas\\data")).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().is_ok_and(|f| f.is_file()) {
            continue;
        }
        fs::copy(entry.path(), out_dir.join(entry.file_name())).unwrap();
    }
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
