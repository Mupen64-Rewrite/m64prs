use std::ffi::CString;

use gtk::{gio::{ActionMap, MenuModel, SimpleAction}, glib::GString, prelude::*, Application};

pub mod actions {
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

fn app_id(s: &str) -> String {
    const PREFIX: &str = "app.";

    let mut r = String::with_capacity(PREFIX.len() + s.len());
    r.push_str(PREFIX);
    r.push_str(s);
    r
}

fn setup_menu_actions(map: &impl IsA<ActionMap>) {
    use crate::views::macros::action;
    use actions::*;

    let action = gtk::gio::SimpleAction::new("foo", None);

    action::simple!(map[file::OPEN_ROM] => |_| println!("Open ROM..."));

    action::simple!(map[file::CLOSE_ROM] => |_| println!("Closing ROM..."));
    action::simple!(map[file::RESET_ROM] => |_| println!("Resetting ROM..."));

    action::simple!(map[emu::PAUSE_RESUME] => |_| println!("Toggle pause..."));
    action::simple!(map[emu::FRAME_ADVANCE] => |_| println!("Frame advance..."));
    action::simple!(map[emu::LOAD_FROM_FILE] => |_| println!("Load from file..."));
    action::simple!(map[emu::SAVE_TO_FILE] => |_| println!("Save to file..."));
}

fn create_menu() -> MenuModel {
    use crate::views::macros::menu::*;
    use self::actions::*;

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

pub fn init(application: &Application) {
    setup_menu_actions(application);
    application.set_menubar(Some(&create_menu()));
}
