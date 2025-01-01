use std::{path::{Path, PathBuf}, sync::LazyLock};

pub struct InstallDirs {
    // system installation
    pub core_dir: PathBuf,
    pub plugin_dir: PathBuf,
    pub data_dir: PathBuf,
    pub i18n_dir: PathBuf,
}

pub static INSTALL_DIRS: LazyLock<InstallDirs> = LazyLock::new(get_install_dirs);

#[cfg(not(feature = "install-unix"))]
fn get_install_dirs() -> InstallDirs {
    use std::env;

    let own_path = env::current_exe().unwrap();
    let own_dir = own_path.parent().unwrap();

    InstallDirs {
        core_dir: own_dir.to_owned(),
        plugin_dir: own_dir.join("plugin"),
        data_dir: own_dir.join("data"),
        i18n_dir: own_dir.join("i18n"),
    }
}

#[cfg(feature = "install-unix")]
fn get_install_dirs() -> InstallDirs {
    use std::env;

    let own_path = env::current_exe().unwrap();
    let own_parent = own_path.ancestors().nth(2).unwrap();

    InstallDirs {
        core_dir: own_parent.join("lib/m64prs"),
        plugin_dir: own_parent.join("lib/m64prs/plugin"),
        data_dir: own_path.join("share/m64prs"),
        i18n_dir: own_path.join("share/locale"),
    }
}
