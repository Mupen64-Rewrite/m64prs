use relm4::{new_action_group, new_stateful_action, new_stateless_action};

// ACTION DEFINITIONS
// ==========================

new_action_group!(pub AppActions, "app");

new_stateless_action!(pub OpenRomAction, AppActions, "file.rom_open");
new_stateless_action!(pub CloseRomAction, AppActions, "file.rom_close");

new_stateful_action!(pub TogglePauseAction, AppActions, "file.enable_pause", (), bool);
