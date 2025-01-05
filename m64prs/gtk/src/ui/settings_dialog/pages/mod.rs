mod emu;
mod plugins;
mod shortcuts;

macro_rules! use_pages {
    {$($page:ty),* $(,)?} => {
        $(pub(super) use $page;)*

        pub(super) fn init_pages() {
            $(<$page as glib::prelude::StaticTypeExt>::ensure_type();)*
        }
    };
}

// pub(super) use emu::EmuPage;
// pub(super) use plugins::PluginsPage;
// pub(super) use shortcuts::ShortcutsPage;

use_pages! {
    emu::EmuPage,
    plugins::PluginsPage,
    shortcuts::ShortcutsPage,
}