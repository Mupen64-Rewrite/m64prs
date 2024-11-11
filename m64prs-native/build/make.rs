use std::{ffi::OsStr, path::Path, process::Command, sync::LazyLock};

pub fn make<I, S>(dir: &Path, make_args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    static COMPILEDB_AVAILABLE: LazyLock<bool> =
        LazyLock::new(|| which::which("compiledb").is_ok());

    let mut cmd = if *COMPILEDB_AVAILABLE {
        let mut builder = Command::new("compiledb");
        builder.arg("make");
        builder
    } else {
        Command::new("make")
    };

    let cmd = cmd
        .args(make_args)
        .current_dir(dir)
        .stdout(os_pipe::dup_stderr().unwrap())
        .status()
        .expect("make exec failed");
    assert!(cmd.success());
}
