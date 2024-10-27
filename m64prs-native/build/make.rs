use std::{path::Path, process::Command, sync::LazyLock};

pub fn make(dir: &Path) {
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
        .args(["all"])
        .current_dir(dir)
        .stdout(os_pipe::dup_stderr().unwrap())
        .status()
        .expect("make exec failed");
    assert!(cmd.success());
}
