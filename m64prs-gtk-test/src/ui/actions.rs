use std::{path::PathBuf, sync::LazyLock};

use crate::ui::UI_GLOBALS;
use gtk::{
    gio::{self, ActionMap, ListStore},
    glib,
    prelude::*,
    FileDialog, FileFilter,
};
use send_wrapper::SendWrapper;

pub mod ids {
    pub mod file {
        pub const OPEN_ROM: &str = "file.open_rom";
        pub const CLOSE_ROM: &str = "file.close_rom";
        pub const RESET_ROM: &str = "file.reset_rom";
    }

    pub mod emu {
        pub const PAUSE_RESUME: &str = "emu.toggle_pause";
        pub const FRAME_ADVANCE: &str = "emu.frame_advance";

        pub const SAVE_TO_FILE: &str = "emu.save_file";
        pub const LOAD_FROM_FILE: &str = "emu.load_file";
    }
}

fn file_filter<const N: usize>(label: &str, patterns: [&str; N]) -> FileFilter {
    let name = format!("{} ({})", label, patterns.join(", "));
    let filter = FileFilter::new();
    filter.set_name(Some(&name));
    for pattern in patterns {
        filter.add_pattern(pattern);
    }
    filter
}

static OPEN_ROM_FILTERS: LazyLock<SendWrapper<ListStore>> = LazyLock::new(|| {
    SendWrapper::new(
        [file_filter("N64 ROMS", ["*.n64", "*.v64", "*.z64"])]
            .into_iter()
            .collect(),
    )
});

pub fn register_actions(map: &impl IsA<ActionMap>) {
    use crate::ui::macros::action;
    use ids::*;

    action::simple!(map[file::OPEN_ROM] => async |_| {
        let state = UI_GLOBALS.get().unwrap();

        let dialog_params = FileDialog::builder()
            .title("Open ROM...")
            .modal(true)
            .filters(&**OPEN_ROM_FILTERS)
            .build();

        let path = match dialog_params.open_future(Some(&state.main_window)).await {
            Ok(file) => file.path().unwrap(),
            Err(_) => return
        };

        println!("Opening ROM at {:?}", path);
    });

    action::simple!(map[file::CLOSE_ROM] => |_| println!("Closing ROM..."));
    action::simple!(map[file::RESET_ROM] => |_| println!("Resetting ROM..."));

    action::simple!(map[emu::PAUSE_RESUME] => |_| println!("Toggle pause..."));
    action::simple!(map[emu::FRAME_ADVANCE] => |_| println!("Frame advance..."));
    action::simple!(map[emu::LOAD_FROM_FILE] => |_| println!("Load from file..."));
    action::simple!(map[emu::SAVE_TO_FILE] => |_| println!("Save to file..."));
}
