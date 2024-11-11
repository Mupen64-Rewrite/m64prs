use relm4::{new_action_group, new_stateful_action, new_stateless_action};

// ACTION DEFINITIONS
// ==========================

new_action_group!(pub FileActions, "app.file");

new_stateless_action!(pub RomOpenAction, FileActions, "rom_open");
new_stateless_action!(pub RomCloseAction, FileActions, "rom_close");

new_action_group!(pub EmuActions, "app.emu");

new_stateful_action!(pub TogglePauseAction, EmuActions, "enable_pause", (), bool);
