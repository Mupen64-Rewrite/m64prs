use std::{cell::Cell, fmt::Debug};

use m64prs_sys::EmuState;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    new_action_group, new_stateful_action, new_stateless_action, Sender,
};

use crate::{old_ui::{core::CoreRequest, dialogs::movie::MovieDialogMode}, utils::actions::RelmActionStateExt};

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

new_stateless_action!(pub NewMovieAction, AppActionsGroup, "vcr.new_movie");
new_stateless_action!(pub LoadMovieAction, AppActionsGroup, "vcr.load_movie");
new_stateless_action!(pub SaveMovieAction, AppActionsGroup, "vcr.save_movie");
new_stateless_action!(pub DiscardMovieAction, AppActionsGroup, "vcr.discard_movie");
new_stateful_action!(pub ToggleReadOnlyAction, AppActionsGroup, "vcr.toggle_read_only", (), bool);

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
    load_file: RelmAction<LoadFileAction>,

    new_movie: RelmAction<NewMovieAction>,
    load_movie: RelmAction<LoadMovieAction>,
    save_movie: RelmAction<SaveMovieAction>,
    discard_movie: RelmAction<DiscardMovieAction>,
    toggle_read_only: RelmAction<ToggleReadOnlyAction>,
}

impl AppActions {
    pub(super) fn new(sender: &Sender<main::Message>) -> Self {
        /// Adds an action to a group and returns it.
        macro_rules! a {
            ($group:ident, $action:expr) => {{
                let a = $action;
                $group.add_action(a.clone());
                a
            }};
        }
        /// Handy macro for an action that just sends a message.
        macro_rules! message {
            ($target:ident => $msg:expr, state: $state:expr) => {{
                let act = ::relm4::actions::RelmAction::new_stateful_with_target_value(&($state), {
                    let sender = sender.clone();
                    move |_, _, $target| sender.emit($msg)
                });
                act.set_enabled(false);
                act
            }};
            ($msg:expr, state: $state:expr) => {{
                let act = ::relm4::actions::RelmAction::new_stateful(&($state), {
                    let sender = sender.clone();
                    move |_, _| sender.emit($msg)
                });
                act.set_enabled(false);
                act
            }};
            ($msg:expr) => {{
                let act = ::relm4::actions::RelmAction::new_stateless({
                    let sender = sender.clone();
                    move |_| sender.emit($msg)
                });
                act.set_enabled(false);
                act
            }};
        }

        let mut group = RelmActionGroup::<AppActionsGroup>::new();
        let inst = Self {
            emu_state: Cell::new(EmuState::Stopped),
            io_state: Cell::new(false),

            open_rom: a!(group, message!(main::Message::ShowOpenRomDialog)),
            close_rom: a!(
                group,
                message!(main::Message::ForwardToCore(CoreRequest::StopRom))
            ),

            toggle_pause: a!(
                group,
                message!(main::Message::ForwardToCore(CoreRequest::TogglePause), state: false)
            ),
            frame_advance: a!(
                group,
                message!(main::Message::ForwardToCore(CoreRequest::FrameAdvance))
            ),
            reset_rom: a!(
                group,
                message!(main::Message::ForwardToCore(CoreRequest::Reset(false)))
            ),

            save_slot: a!(
                group,
                message!(main::Message::ForwardToCore(CoreRequest::SaveSlot))
            ),
            load_slot: a!(
                group,
                message!(main::Message::ForwardToCore(CoreRequest::LoadSlot))
            ),
            set_save_slot: a!(
                group,
                message!(slot => main::Message::ForwardToCore(CoreRequest::SetSaveSlot(slot)), state: 1)
            ),
            save_file: a!(group, message!(main::Message::ShowSaveFileDialog)),
            load_file: a!(group, message!(main::Message::ShowLoadFileDialog)),

            new_movie: a!(group, message!(main::Message::ShowMovieDialog(MovieDialogMode::New))),
            load_movie: a!(group, message!(main::Message::ShowMovieDialog(MovieDialogMode::Load))),

            save_movie: a!(group, message!(main::Message::NoOp)),
            discard_movie: a!(group, message!(main::Message::NoOp)),

            toggle_read_only: a!(group, message!(main::Message::ForwardToCore(CoreRequest::ToggleVcrReadOnly), state: false)),
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

        self.new_movie.set_enabled(is_running);
        self.load_movie.set_enabled(is_running);
        self.save_movie.set_enabled(is_running);
        self.discard_movie.set_enabled(is_running);
        self.toggle_read_only.set_enabled(is_running);
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

    pub(super) fn set_vcr_read_only(&self, read_only: bool) {
        self.toggle_read_only.set_state(read_only);
    }
}

impl Debug for AppActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppActions { <private data> }")
    }
}
