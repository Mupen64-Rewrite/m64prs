use crate::utils::dirs::INSTALL_DIRS;

pub fn setup_gettext() {
    gettextrs::bindtextdomain("m64prs", &INSTALL_DIRS.i18n_dir);
    gettextrs::textdomain("m64prs");
}