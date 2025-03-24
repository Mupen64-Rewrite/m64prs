mod emu;
mod plugins;
mod shortcuts;

use glib::types::StaticTypeExt;
use m64prs_core::Core;

pub(super) use emu::EmuPage;
pub(super) use plugins::PluginsPage;
pub(super) use shortcuts::ShortcutsPage;

/// Ensures all page classes are initialized.
pub(super) fn ensure_types() {
    EmuPage::ensure_type();
    PluginsPage::ensure_type();
    ShortcutsPage::ensure_type();
}

/// Performs setup on the initial configuration values.
pub fn init_config(core: &mut Core) {
    plugins::init_config(core);
    shortcuts::init_config(core);
}
