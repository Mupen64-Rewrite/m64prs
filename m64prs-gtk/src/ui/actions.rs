use std::fmt::Debug;

use m64prs_sys::EmuState;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    new_action_group, new_stateful_action, new_stateless_action, ComponentSender,
};

use super::main;

// ACTION DEFINITIONS
// ==========================

new_action_group!(pub AppActionGroup, "app");

new_stateless_action!(pub OpenRomAction, AppActionGroup, "file.rom_open");
new_stateless_action!(pub CloseRomAction, AppActionGroup, "file.rom_close");

new_stateful_action!(pub TogglePauseAction, AppActionGroup, "emu.toggle_pause", (), bool);

pub(super) struct AppActions {
    open_rom: RelmAction<OpenRomAction>,
    close_rom: RelmAction<CloseRomAction>,
}

impl AppActions {
    pub(super) fn new(sender: &ComponentSender<main::Model>) -> Self {
        macro_rules! action {
            ($group:ident, $action:expr) => {{
                let a = $action;
                $group.add_action(a.clone());
                a
            }};
        }
        macro_rules! send_message {
            ($msg:expr) => {{
                let sender = sender.clone();
                move |_| sender.input($msg)
            }};
        }

        let mut group = RelmActionGroup::<AppActionGroup>::new();
        let inst = Self {
            open_rom: action!(group, RelmAction::new_stateless(send_message!(
                main::Message::MenuRomOpen
            ))),
            close_rom: action!(group, RelmAction::new_stateless(send_message!(
                main::Message::MenuRomClose
            ))),
        };
        group.register_for_main_application();

        inst
    }

    pub(super) fn set_mupen_state(&self, state: Option<EmuState>) {
        log::debug!("set_mupen_state: {:?}", state);

        self.open_rom.set_enabled(matches!(state, Some(EmuState::Stopped)));
        self.close_rom.set_enabled(matches!(state, Some(EmuState::Running | EmuState::Paused)));
    }
}

impl Debug for AppActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppActions { <private data> }")
    }
}