use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

mod dirs;

pub fn setup_cargo_reruns() {
    fn emit(path: &Path) {
        println!("cargo::rerun-if-changed={}", path.display())
    }

    println!("cargo::rerun-if-changed=build/");
    #[cfg(windows)]
    println!("cargo::rerun-if-changed=m64prs-build-win.py");
    #[cfg(unix)]
    println!("cargo::rerun-if-changed=m64prs-build-unix.py");
    // mupen64plus-core-tas
    {
        let core_dir = Path::new(dirs::M64P_CORE_DIR);
        for entry in walkdir::WalkDir::new(&core_dir.join("src")) {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().is_file()
                && !path
                    .components()
                    .any(|comp| comp.as_os_str() == OsStr::new("asm_defines"))
            {
                emit(path);
            }
        }
        #[cfg(windows)]
        {
            let project_dir = &core_dir.join("projects/msvc");
            emit(&project_dir.join("mupen64plus-core.vcxproj"));
            emit(&project_dir.join("mupen64plus-core.vcxproj.filters"));
        }
        #[cfg(unix)]
        {
            let project_dir = &core_dir.join("projects/unix");
            emit(&project_dir.join("Makefile"));
        }
    }
}

#[cfg(windows)]
fn compile_m64p_deps(_out_dir: &Path) {
    use std::process::{Command, Stdio};

    let root_dir = PathBuf::from(dirs::ROOT_DIR);

    let cmd = Command::new("python3")
        .arg(root_dir.join("m64prs-build-win.py"))
        .arg("build")
        .stdout(os_pipe::dup_stderr().unwrap())
        .status()
        .expect("script invoke failed");

    assert!(cmd.success());
}

#[cfg(unix)]
fn compile_m64p_deps(_out_dir: &Path) {
    use std::process::{Command, Stdio};

    let root_dir = PathBuf::from(dirs::ROOT_DIR);

    let cmd = Command::new("python3")
        .arg(root_dir.join("m64prs-build-unix.py"))
        .arg("build")
        .stdout(os_pipe::dup_stderr().unwrap())
        .status()
        .expect("script invoke failed");

    assert!(cmd.success());
}

fn main() {
    // ./target contains all native outputs
    let out_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("target");
    fs::create_dir_all(&out_dir).unwrap();

    setup_cargo_reruns();
    compile_m64p_deps(&out_dir);
}
