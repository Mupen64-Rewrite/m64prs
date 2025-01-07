mod emu;
mod plugins;
mod shortcuts;

pub(super) use emu::EmuPage;
use glib::types::StaticTypeExt;
pub(super) use plugins::PluginsPage;
pub(super) use shortcuts::ShortcutsPage;

pub(super) fn init_pages() {
    EmuPage::ensure_type();
    PluginsPage::ensure_type();
    ShortcutsPage::ensure_type();
}
