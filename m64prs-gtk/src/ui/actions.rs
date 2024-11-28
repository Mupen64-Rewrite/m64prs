use std::fmt::Debug;

use m64prs_sys::EmuState;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    new_action_group, new_stateful_action, new_stateless_action, ComponentSender,
};

use crate::utils::actions::RelmActionStateExt;

use super::main;

// ACTION DEFINITIONS
// ==========================

new_action_group!(pub AppActionGroup, "app");

new_stateless_action!(pub OpenRomAction, AppActionGroup, "file.open_rom");
new_stateless_action!(pub CloseRomAction, AppActionGroup, "file.close_rom");

new_stateful_action!(pub TogglePauseAction, AppActionGroup, "emu.toggle_pause", (), bool);
new_stateless_action!(pub FrameAdvanceAction, AppActionGroup, "emu.frame_advance");
new_stateless_action!(pub ResetRomAction, AppActionGroup, "emu.reset_rom");

pub(super) struct AppActions {
    open_rom: RelmAction<OpenRomAction>,
    close_rom: RelmAction<CloseRomAction>,
    toggle_pause: RelmAction<TogglePauseAction>,
    frame_advance: RelmAction<FrameAdvanceAction>,
    reset_rom: RelmAction<ResetRomAction>,
}

impl AppActions {
    pub(super) fn new(sender: &ComponentSender<main::Model>) -> Self {
        macro_rules! a {
            ($group:ident, $action:expr) => {{
                let a = $action;
                $group.add_action(a.clone());
                a
            }};
        }
        macro_rules! send_message {
            ($msg:expr) => {
                ::relm4::actions::RelmAction::new_stateless({
                    let sender = sender.clone();
                    move |_| sender.input($msg)
                })
            };
        }

        let mut group = RelmActionGroup::<AppActionGroup>::new();
        let inst = Self {
            open_rom: a!(group, send_message!(main::Message::MenuOpenRom)),
            close_rom: a!(group, send_message!(main::Message::MenuCloseRom)),
            toggle_pause: a!(
                group,
                RelmAction::new_stateful(&false, {
                    let sender = sender.clone();
                    move |_, _| sender.input(main::Message::MenuTogglePause)
                })
            ),
            frame_advance: a!(group, send_message!(main::Message::MenuFrameAdvance)),
            reset_rom: a!(group, send_message!(main::Message::MenuResetRom)),
        };
        group.register_for_main_application();

        inst
    }

    pub(super) fn set_mupen_state(&self, state: EmuState) {
        log::debug!("set_mupen_state: {:?}", state);

        let is_stopped = matches!(state, EmuState::Stopped);
        let is_running = matches!(state, EmuState::Running | EmuState::Paused);
        let is_paused = matches!(state, EmuState::Paused);

        self.open_rom.set_enabled(is_stopped);
        self.close_rom.set_enabled(is_running);

        self.toggle_pause.set_enabled(is_running);
        self.toggle_pause.set_state(is_paused);
        self.frame_advance.set_enabled(is_running);
        self.reset_rom.set_enabled(is_running);
    }
}

impl Debug for AppActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppActions { <private data> }")
    }
}
