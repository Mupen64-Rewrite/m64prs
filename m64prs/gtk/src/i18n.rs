use crate::utils::paths::INSTALL_DIRS;

pub fn setup_gettext() {
    gettextrs::bindtextdomain("m64prs", &INSTALL_DIRS.i18n_dir)
        .expect("Failed to load translation files!");
    gettextrs::textdomain("m64prs").expect("Failed to set gettext domain!");
}
