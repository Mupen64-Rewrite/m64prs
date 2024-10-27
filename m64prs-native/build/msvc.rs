use std::{
    env,
    ffi::OsStr,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::LazyLock,
};

pub fn vswhere(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Option<String> {
    static VSWHERE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        let mut program_files_x86: PathBuf = env::var_os("ProgramFiles(x86)").unwrap().into();
        program_files_x86.push("Microsoft Visual Studio\\Installer\\vswhere.exe");
        program_files_x86
    });

    let cmd = Command::new(VSWHERE_PATH.as_path())
        .args(args)
        .output()
        .expect("vswhere exec failed; is VS installed?");

    if !cmd.status.success() {
        return None;
    }

    Some(String::from_utf8(cmd.stdout).unwrap())
}

pub fn vs_dev_env(arch: &str) -> Vec<(String, String)> {
    use std::os::windows::process::CommandExt;

    let devenv_path = {
        let vswhere_query: PathBuf = vswhere(["-latest", "-property", "installationPath"])
            .unwrap()
            .trim()
            .into();
        vswhere_query.join("Common7\\Tools\\VsDevCmd.bat")
    };

    let mut cmd = Command::new("cmd.exe")
        .args(["/s", "/c"])
        .raw_arg(format!(
            r#"""{}" -no_logo -arch={} && set""#,
            devenv_path.to_str().unwrap(),
            arch
        ))
        .stdout(Stdio::piped())
        .spawn()
        .expect("VsDevCmd.bat exec failed");

    // read output line by line, storing env variables
    let br = BufReader::new(cmd.stdout.as_mut().unwrap());
    let mut result = Vec::<(String, String)>::new();
    for line in br.lines() {
        let line = line.unwrap();
        let split_pos = line.find('=').expect("windows env format broke");
        result.push((
            line[..split_pos].to_owned(),
            line[(split_pos + 1)..].to_owned(),
        ));
    }

    let cmd = cmd.wait().expect("VsDevCmd.bat wait failed");
    assert!(cmd.success());
    result
}

pub fn msbuild(
    env: &[(String, String)],
    sln_file: &Path,
    target_dir: &Path,
    config: &str,
    platform: &str,
) {
    static MSBUILD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        vswhere([
            "-latest",
            "-requires",
            "Microsoft.Component.MSBuild",
            "-find",
            r"MSBuild\**\Bin\MSBuild.exe",
        ])
        .expect("msbuild not installed")
        .trim()
        .into()
    });

    let cmd = Command::new(MSBUILD_PATH.as_path())
        .arg(format!("/p:Configuration={}", config))
        .arg(format!("/p:Platform={}", platform))
        .arg(format!("/p:OutDir={}", target_dir.to_str().unwrap()))
        .arg(sln_file)
        .envs(env.iter().map(|pair| (&pair.0, &pair.1)))
        .stdout(os_pipe::dup_stderr().unwrap())
        .status()
        .expect("msbuild invoke failed");

    assert!(cmd.success());
}
