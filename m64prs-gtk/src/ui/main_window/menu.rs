use gtk::prelude::*;

use crate::utils::actions::{ActionGroupTypedExt, BaseAction, StateAction, StateParamAction};

pub fn load_menu() -> gio::MenuModel {
    const UI_XML: &str = gtk::gtk4_macros::include_blueprint!("src/ui/main_window/menu.blp");
    gtk::Builder::from_string(UI_XML)
        .object::<gio::MenuModel>("root")
        .expect("menu.blp should contain object `root`")
}

struct AppActions {
    open_rom: BaseAction,
    close_rom: BaseAction,

    toggle_pause: StateAction<bool>,
    frame_advance: BaseAction,
    reset_rom: BaseAction,

    save_slot: BaseAction,
    load_slot: BaseAction,
    set_save_slot: StateParamAction<u8, u8>,
    save_file: BaseAction,
    load_file: BaseAction,

    new_movie: BaseAction,
    load_movie: BaseAction,
    save_movie: BaseAction,
    discard_movie: BaseAction,
    toggle_read_only: StateAction<bool>,
}
impl AppActions {
    fn register_to(&self, map: &impl IsA<gio::ActionMap>) {
        macro_rules! register_all_actions {
            ($($names:ident),* $(,)?) => {
                {
                    $(map.register_action(&self.$names);)*
                }
            };
        }
        register_all_actions!(
            open_rom, close_rom,
        );
    }
}
