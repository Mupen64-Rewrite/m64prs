use std::{cell::Cell, fmt::Debug};

use m64prs_sys::EmuState;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    new_action_group, new_stateful_action, new_stateless_action, ComponentSender,
};

use crate::utils::actions::RelmActionStateExt;

use super::main;

// ACTION DEFINITIONS
// ==========================

new_action_group!(pub AppActionsGroup, "app");

new_stateless_action!(pub OpenRomAction, AppActionsGroup, "file.open_rom");
new_stateless_action!(pub CloseRomAction, AppActionsGroup, "file.close_rom");

new_stateful_action!(pub TogglePauseAction, AppActionsGroup, "emu.toggle_pause", (), bool);
new_stateless_action!(pub FrameAdvanceAction, AppActionsGroup, "emu.frame_advance");
new_stateless_action!(pub ResetRomAction, AppActionsGroup, "emu.reset_rom");

new_stateless_action!(pub SaveSlotAction, AppActionsGroup, "emu.save_slot");
new_stateless_action!(pub LoadSlotAction, AppActionsGroup, "emu.load_slot");
new_stateful_action!(pub SetSaveSlotAction, AppActionsGroup, "emu.set_save_slot", u8, u8);
new_stateless_action!(pub SaveFileAction, AppActionsGroup, "emu.save_file");
new_stateless_action!(pub LoadFileAction, AppActionsGroup, "emu.load_file");

pub(super) struct AppActions {
    emu_state: Cell<EmuState>,
    io_state: Cell<bool>,

    open_rom: RelmAction<OpenRomAction>,
    close_rom: RelmAction<CloseRomAction>,
    toggle_pause: RelmAction<TogglePauseAction>,
    frame_advance: RelmAction<FrameAdvanceAction>,
    reset_rom: RelmAction<ResetRomAction>,
    save_slot: RelmAction<SaveSlotAction>,
    load_slot: RelmAction<LoadSlotAction>,
    set_save_slot: RelmAction<SetSaveSlotAction>,
    save_file: RelmAction<SaveFileAction>,
    load_file: RelmAction<LoadFileAction>
}

impl AppActions {
    pub(super) fn new(sender: &ComponentSender<main::Model>) -> Self {
        /// Adds an action to a group and returns it.
        macro_rules! a {
            ($group:ident, $action:expr) => {{
                let a = $action;
                $group.add_action(a.clone());
                a
            }};
        }
        /// Handy macro for an action that just sends a message.
        macro_rules! send_message {
            ($msg:expr) => {
                ::relm4::actions::RelmAction::new_stateless({
                    let sender = sender.clone();
                    move |_| sender.input($msg)
                })
            };
            ($msg:expr, state: $state:expr) => {
                ::relm4::actions::RelmAction::new_stateful($state, {
                    let sender = sender.clone();
                    move |_, _| sender.input($msg)
                })
            };
        }

        let mut group = RelmActionGroup::<AppActionsGroup>::new();
        let inst = Self {
            emu_state: Cell::new(EmuState::Stopped),
            io_state: Cell::new(false),

            open_rom: a!(group, send_message!(main::Message::MenuOpenRom)),
            close_rom: a!(group, send_message!(main::Message::MenuCloseRom)),
            toggle_pause: a!(
                group,
                send_message!(main::Message::MenuTogglePause, state: &false)
            ),
            frame_advance: a!(group, send_message!(main::Message::MenuFrameAdvance)),
            reset_rom: a!(group, send_message!(main::Message::MenuResetRom)),
            save_slot: a!(group, send_message!(main::Message::MenuSaveSlot)),
            load_slot: a!(group, send_message!(main::Message::MenuLoadSlot)),
            set_save_slot: a!(
                group,
                RelmAction::new_stateful_with_target_value(&1u8, {
                    let sender = sender.clone();
                    move |_, _, slot| sender.input(main::Message::MenuSetSaveSlot(slot))
                })
            ),
            save_file: a!(group, send_message!(main::Message::MenuSaveFile)),
            load_file: a!(group, send_message!(main::Message::MenuLoadFile)),
        };
        group.register_for_main_application();

        inst
    }

    pub(super) fn set_core_state(&self, emu_state: EmuState) {
        self.emu_state.set(emu_state);

        let is_stopped = matches!(emu_state, EmuState::Stopped);
        let is_running = matches!(emu_state, EmuState::Running | EmuState::Paused);
        let is_paused = matches!(emu_state, EmuState::Paused);

        let is_save_ok = matches!(
            (emu_state, self.io_state.get()),
            (EmuState::Running | EmuState::Paused, false)
        );

        self.open_rom.set_enabled(is_stopped);
        self.close_rom.set_enabled(is_running);

        self.toggle_pause.set_enabled(is_running);
        self.toggle_pause.set_state(is_paused);
        self.frame_advance.set_enabled(is_running);
        self.reset_rom.set_enabled(is_running);

        self.save_slot.set_enabled(is_save_ok);
        self.load_slot.set_enabled(is_save_ok);
        self.set_save_slot.set_enabled(is_save_ok);
        self.save_file.set_enabled(is_save_ok);
        self.load_file.set_enabled(is_save_ok);
    }

    pub(super) fn set_core_io_state(&self, io_state: bool) {
        self.io_state.set(io_state);

        let is_save_ok = matches!(
            (self.emu_state.get(), io_state),
            (EmuState::Running | EmuState::Paused, false)
        );

        self.save_slot.set_enabled(is_save_ok);
        self.load_slot.set_enabled(is_save_ok);
        self.set_save_slot.set_enabled(is_save_ok);
        self.save_file.set_enabled(is_save_ok);
        self.load_file.set_enabled(is_save_ok);
    }

    pub(super) fn set_core_savestate_slot(&self, slot: u8) {
        self.set_save_slot.set_state(slot);
    }
}

impl Debug for AppActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppActions { <private data> }")
    }
}
