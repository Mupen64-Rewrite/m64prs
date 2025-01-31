//! [`decan`]-compatible [`SymbolGroup`] implementations for the
//! Mupen64Plus API.

use decan::{non_null, SymbolGroup};

use crate::{ext::ptr_M64PRS_UseFrontendHandle, types::*};

#[derive(SymbolGroup)]
pub struct FullCoreApi {
    #[subgroup]
    pub base: CoreBaseApi,
    #[subgroup]
    pub config: CoreConfigApi,
    #[subgroup]
    pub tas: CoreTasApi,
}

#[derive(SymbolGroup)]
pub struct PluginCoreApi {
    #[subgroup]
    pub config: CoreConfigApi,
    #[subgroup]
    pub tas: CoreTasApi,
}

#[derive(SymbolGroup)]
pub struct CoreBaseApi {
    #[symbol = "PluginGetVersion"]
    pub get_version: non_null!(ptr_PluginGetVersion),
    #[symbol = "CoreErrorMessage"]
    pub error_message: non_null!(ptr_CoreErrorMessage),
    #[symbol = "CoreStartup"]
    pub startup: non_null!(ptr_CoreStartup),
    #[symbol = "CoreShutdown"]
    pub shutdown: non_null!(ptr_CoreShutdown),
    #[symbol = "CoreAttachPlugin"]
    pub attach_plugin: non_null!(ptr_CoreAttachPlugin),
    #[symbol = "CoreDetachPlugin"]
    pub detach_plugin: non_null!(ptr_CoreDetachPlugin),
    #[symbol = "CoreDoCommand"]
    pub do_command: non_null!(ptr_CoreDoCommand),
    #[symbol = "CoreOverrideVidExt"]
    pub override_vidext: non_null!(ptr_CoreOverrideVidExt),
}

#[derive(SymbolGroup)]
pub struct CoreConfigApi {
    // DIRECTORIES
    // =================
    #[symbol = "ConfigGetSharedDataFilepath"]
    pub shared_data_filepath: non_null!(ptr_ConfigGetSharedDataFilepath),

    // DISCOVERY
    // =================
    #[symbol = "ConfigListSections"]
    pub list_sections: non_null!(ptr_ConfigListSections),
    #[symbol = "ConfigOpenSection"]
    pub open_section: non_null!(ptr_ConfigOpenSection),
    #[symbol = "ConfigListParameters"]
    pub list_parameters: non_null!(ptr_ConfigListParameters),

    // SECTION MODIFIERS
    // =================
    #[symbol = "ConfigDeleteSection"]
    pub delete_section: non_null!(ptr_ConfigDeleteSection),
    #[symbol = "ConfigSaveFile"]
    pub save_file: non_null!(ptr_ConfigSaveFile),
    #[symbol = "ConfigSaveSection"]
    pub save_section: non_null!(ptr_ConfigSaveSection),
    #[symbol = "ConfigRevertChanges"]
    pub revert_section: non_null!(ptr_ConfigRevertChanges),

    // GETTERS
    // ================
    #[symbol = "ConfigGetParameterHelp"]
    pub get_parameter_help: non_null!(ptr_ConfigGetParameterHelp),
    #[symbol = "ConfigGetParameterType"]
    pub get_parameter_type: non_null!(ptr_ConfigGetParameterType),
    #[symbol = "ConfigGetParamInt"]
    pub get_param_int: non_null!(ptr_ConfigGetParamInt),
    #[symbol = "ConfigGetParamFloat"]
    pub get_param_float: non_null!(ptr_ConfigGetParamFloat),
    #[symbol = "ConfigGetParamBool"]
    pub get_param_bool: non_null!(ptr_ConfigGetParamBool),
    #[symbol = "ConfigGetParamString"]
    pub get_param_string: non_null!(ptr_ConfigGetParamString),

    // SETTERS
    // ==============
    #[symbol = "ConfigSetParameter"]
    pub set_parameter: non_null!(ptr_ConfigSetParameter),
    #[symbol = "ConfigSetParameterHelp"]
    pub set_parameter_help: non_null!(ptr_ConfigSetParameterHelp),

    // DEFAULT SETTERS
    // ==============
    #[symbol = "ConfigSetDefaultInt"]
    pub set_default_int: non_null!(ptr_ConfigSetDefaultInt),
    #[symbol = "ConfigSetDefaultFloat"]
    pub set_default_float: non_null!(ptr_ConfigSetDefaultFloat),
    #[symbol = "ConfigSetDefaultBool"]
    pub set_default_bool: non_null!(ptr_ConfigSetDefaultBool),
    #[symbol = "ConfigSetDefaultString"]
    pub set_default_string: non_null!(ptr_ConfigSetDefaultString),
}

#[derive(SymbolGroup)]
pub struct CoreTasApi {
    #[symbol = "CoreTAS_SetInputHandler"]
    pub set_input_handler: non_null!(ptr_CoreTAS_SetInputHandler),
    #[symbol = "CoreTAS_SetAudioHandler"]
    pub set_audio_handler: non_null!(ptr_CoreTAS_SetAudioHandler),
    #[symbol = "CoreTAS_SetAudioTapEnabled"]
    pub set_audio_tap_enabled: non_null!(ptr_CoreTAS_SetAudioTapEnabled),
    #[symbol = "CoreTAS_SetSavestateHandler"]
    pub set_savestate_handler: non_null!(ptr_CoreTAS_SetSavestateHandler),
}

#[derive(SymbolGroup)]
pub struct BasePluginApi {
    #[symbol = "PluginGetVersion"]
    pub get_version: non_null!(ptr_PluginGetVersion),
    #[symbol = "PluginStartup"]
    pub startup: non_null!(ptr_PluginStartup),
    #[symbol = "PluginShutdown"]
    pub shutdown: non_null!(ptr_PluginShutdown),
}

#[derive(SymbolGroup)]
pub struct ExtPluginApi {
    #[symbol = "M64PRS_UseFrontendHandle"]
    pub use_frontend_interface: non_null!(ptr_M64PRS_UseFrontendHandle),
}