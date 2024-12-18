use std::ffi::{c_char, c_float, c_int, c_void};

use decan::{non_null, SymbolGroup};
use dlopen2::wrapper::{WrapperApi, WrapperMultiApi};

use crate::types::*;

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
pub struct CoreBaseApi {
    #[symbol = "PluginGetVersion"]
    pub get_version: non_null!(ptr_PluginGetVersion),
    #[symbol = "CoreErrorMessage"]
    pub error_message: unsafe extern "C" fn(return_code: Error) -> *const c_char,
    #[symbol = "CoreStartup"]
    pub startup: unsafe extern "C" fn(
        api_version: c_int,
        config_path: *const c_char,
        data_path: *const c_char,
        debug_context: *mut c_void,
        debug_callback: unsafe extern "C" fn(
            context: *mut c_void,
            level: c_int,
            message: *const c_char,
        ),
        state_context: *mut c_void,
        state_callback: unsafe extern "C" fn(
            context: *mut c_void,
            param: CoreParam,
            new_value: c_int,
        ),
    ) -> Error,
    #[symbol = "CoreShutdown"]
    pub shutdown: unsafe extern "C" fn() -> Error,
    #[symbol = "CoreAttachPlugin"]
    pub attach_plugin:
        unsafe extern "C" fn(plugin_type: PluginType, plugin_lib_handle: DynlibHandle) -> Error,
    #[symbol = "CoreDetachPlugin"]
    pub detach_plugin: unsafe extern "C" fn(plugin_type: PluginType) -> Error,
    #[symbol = "CoreDoCommand"]
    pub do_command:
        unsafe extern "C" fn(command: Command, int_param: c_int, ptr_param: *mut c_void) -> Error,
    #[symbol = "CoreOverrideVidExt"]
    pub override_vidext:
        unsafe extern "C" fn(video_function_struct: *mut VideoExtensionFunctions) -> Error,
}

#[derive(SymbolGroup)]
pub struct CoreConfigApi {
    // DIRECTORIES
    // =================
    #[symbol = "ConfigGetSharedDataFilepath"]
    pub shared_data_filepath: unsafe extern "C" fn(name: *const c_char) -> *const c_char,

    // DISCOVERY
    // =================
    #[symbol = "ConfigListSections"]
    pub list_sections: unsafe extern "C" fn(
        context: *mut c_void,
        callback: unsafe extern "C" fn(context: *mut c_void, section_name: *const c_char),
    ) -> Error,
    #[symbol = "ConfigOpenSection"]
    pub open_section: unsafe extern "C" fn(name: *const c_char, handle: *mut Handle) -> Error,
    #[symbol = "ConfigListParameters"]
    pub list_parameters: unsafe extern "C" fn(
        handle: Handle,
        context: *mut c_void,
        callback: unsafe extern "C" fn(context: *mut c_void, name: *const c_char, ptype: ConfigType),
    ) -> Error,

    // SECTION MODIFIERS
    // =================
    #[symbol = "ConfigDeleteSection"]
    pub delete_section: unsafe extern "C" fn(name: *const c_char) -> Error,
    #[symbol = "ConfigSaveFile"]
    pub save_file: unsafe extern "C" fn() -> Error,
    #[symbol = "ConfigSaveSection"]
    pub save_section: unsafe extern "C" fn(name: *const c_char) -> Error,
    #[symbol = "ConfigRevertChanges"]
    pub revert_section: unsafe extern "C" fn(name: *const c_char) -> Error,

    // GETTERS
    // ================
    #[symbol = "ConfigGetParameterHelp"]
    pub get_parameter_help:
        unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> *const c_char,
    #[symbol = "ConfigGetParameterType"]
    pub get_parameter_type: unsafe extern "C" fn(
        handle: Handle,
        param_name: *const c_char,
        param_type: *mut ConfigType,
    ) -> Error,
    #[symbol = "ConfigGetParamInt"]
    pub get_param_int: unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> c_int,
    #[symbol = "ConfigGetParamFloat"]
    pub get_param_float: unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> c_float,
    #[symbol = "ConfigGetParamBool"]
    pub get_param_bool: unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> bool,
    #[symbol = "ConfigGetParamString"]
    pub get_param_string:
        unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> *const c_char,

    // SETTERS
    // ==============
    #[symbol = "ConfigSetParameter"]
    pub set_parameter: unsafe extern "C" fn(
        handle: Handle,
        param_name: *const c_char,
        param_type: ConfigType,
        param_value: *const c_void,
    ) -> Error,
    #[symbol = "ConfigSetParameterHelp"]
    pub set_parameter_help: unsafe extern "C" fn(
        handle: Handle,
        param_name: *const c_char,
        param_help: *const c_char,
    ) -> Error,
}

#[derive(SymbolGroup)]
pub struct CoreTasApi {
    #[symbol = "CoreTAS_SetInputHandler"]
    pub set_input_handler: unsafe extern "C" fn(new_input_handler: *const TasInputHandler) -> Error,
    #[symbol = "CoreTAS_SetAudioHandler"]
    pub set_audio_handler: unsafe extern "C" fn(new_audio_handler: *const TasAudioHandler) -> Error,
    #[symbol = "CoreTAS_SetAudioTapEnabled"]
    pub set_audio_tap_enabled: unsafe extern "C" fn(value: bool) -> Error,
    #[symbol = "CoreTAS_SetSavestateHandler"]
    pub set_savestate_handler: unsafe extern "C" fn(new_save_handler: *const TasSaveHandler) -> Error,
}

#[derive(SymbolGroup)]
pub struct BasePluginApi {
    #[symbol = "PluginGetVersion"]
    pub get_version: unsafe extern "C" fn(
        plugin_type: *mut PluginType,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> Error,
    #[symbol = "PluginStartup"]
    pub startup: unsafe extern "C" fn(
        core_lib_handle: DynlibHandle,
        debug_context: *mut c_void,
        debug_callback: unsafe extern "C" fn(
            context: *mut c_void,
            level: c_int,
            message: *const c_char,
        ),
    ) -> Error,
    #[symbol = "PluginShutdown"]
    pub shutdown: unsafe extern "C" fn() -> Error,
}
