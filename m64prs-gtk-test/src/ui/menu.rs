use std::ffi::CString;

use gtk::{gio::{ActionMap, MenuModel, SimpleAction}, glib::GString, prelude::*};

fn app_id(s: &str) -> String {
    const PREFIX: &str = "app.";

    let mut r = String::with_capacity(PREFIX.len() + s.len());
    r.push_str(PREFIX);
    r.push_str(s);
    r
}

pub fn create_menu() -> MenuModel {
    use crate::ui::macros::menu::*;
    use crate::ui::actions::ids::*;

    root!([menu_bar] {
        submenu!([menu_bar, menu] "File" {
            item!([menu] "Open ROM" => &app_id(file::OPEN_ROM));
            item!([menu] "Close ROM" => &app_id(file::CLOSE_ROM));
            item!([menu] "Reset ROM" => &app_id(file::RESET_ROM));
        });
        submenu!([menu_bar, menu] "Emulator" {
            section!([menu, sect] {
                item!([sect] "Pause/Resume" => &app_id(emu::PAUSE_RESUME));
                item!([sect] "Frame Advance" => &app_id(emu::FRAME_ADVANCE));
            });
            section!([menu, sect] {
                item!([sect] "Save to File" => &app_id(emu::SAVE_TO_FILE));
                item!([sect] "Load from File" => &app_id(emu::LOAD_FROM_FILE));
            });
        });
    })
}

