use std::ffi::{c_char, c_float, c_int, c_void};

use dlopen2::wrapper::{WrapperApi, WrapperMultiApi};

use crate::types::*;

#[derive(WrapperMultiApi)]
pub struct FullCoreApi {
    pub base: CoreBaseApi,
    pub config: CoreConfigApi,
    pub tas: CoreTasApi,
}

#[derive(WrapperApi)]
pub struct CoreBaseApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut PluginType,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> Error,
    #[dlopen2_name = "CoreErrorMessage"]
    error_message: unsafe extern "C" fn(return_code: Error) -> *const c_char,
    #[dlopen2_name = "CoreStartup"]
    startup: unsafe extern "C" fn(
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
    #[dlopen2_name = "CoreShutdown"]
    shutdown: unsafe extern "C" fn() -> Error,
    #[dlopen2_name = "CoreAttachPlugin"]
    attach_plugin:
        unsafe extern "C" fn(plugin_type: PluginType, plugin_lib_handle: DynlibHandle) -> Error,
    #[dlopen2_name = "CoreDetachPlugin"]
    detach_plugin: unsafe extern "C" fn(plugin_type: PluginType) -> Error,
    #[dlopen2_name = "CoreDoCommand"]
    do_command:
        unsafe extern "C" fn(command: Command, int_param: c_int, ptr_param: *mut c_void) -> Error,
    #[dlopen2_name = "CoreOverrideVidExt"]
    override_vidext:
        unsafe extern "C" fn(video_function_struct: *mut VideoExtensionFunctions) -> Error,
}

#[derive(WrapperApi)]
pub struct CoreConfigApi {
    // DISCOVERY
    // =================
    #[dlopen2_name = "ConfigListSections"]
    list_sections: unsafe extern "C" fn(
        context: *mut c_void,
        callback: unsafe extern "C" fn(context: *mut c_void, section_name: *const c_char),
    ) -> Error,
    #[dlopen2_name = "ConfigOpenSection"]
    open_section: unsafe extern "C" fn(name: *const c_char, handle: *mut Handle) -> Error,
    #[dlopen2_name = "ConfigListParameters"]
    list_parameters: unsafe extern "C" fn(
        handle: Handle,
        context: *mut c_void,
        callback: unsafe extern "C" fn(context: *mut c_void, name: *const c_char, ptype: ConfigType),
    ) -> Error,

    // SECTION MODIFIERS
    // =================
    #[dlopen2_name = "ConfigDeleteSection"]
    delete_section: unsafe extern "C" fn(name: *const c_char) -> Error,
    #[dlopen2_name = "ConfigSaveFile"]
    save_file: unsafe extern "C" fn() -> Error,
    #[dlopen2_name = "ConfigSaveSection"]
    save_section: unsafe extern "C" fn(name: *const c_char) -> Error,
    #[dlopen2_name = "ConfigRevertChanges"]
    revert_section: unsafe extern "C" fn(name: *const c_char) -> Error,

    // GETTERS
    // ================
    #[dlopen2_name = "ConfigGetParameterHelp"]
    get_parameter_help:
        unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> *const c_char,
    #[dlopen2_name = "ConfigGetParameterType"]
    get_parameter_type: unsafe extern "C" fn(
        handle: Handle,
        param_name: *const c_char,
        param_type: *mut ConfigType,
    ) -> Error,
    #[dlopen2_name = "ConfigGetParamInt"]
    get_param_int: unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> c_int,
    #[dlopen2_name = "ConfigGetParamFloat"]
    get_param_float: unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> c_float,
    #[dlopen2_name = "ConfigGetParamBool"]
    get_param_bool: unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> bool,
    #[dlopen2_name = "ConfigGetParamString"]
    get_param_string:
        unsafe extern "C" fn(handle: Handle, param_name: *const c_char) -> *const c_char,

    // SETTERS
    // ==============
    #[dlopen2_name = "ConfigSetParameter"]
    set_parameter: unsafe extern "C" fn(
        handle: Handle,
        param_name: *const c_char,
        param_type: ConfigType,
        param_value: *const c_void,
    ) -> Error,
    #[dlopen2_name = "ConfigSetParameterHelp"]
    set_parameter_help:
        unsafe extern "C" fn(handle: Handle, param_name: *const c_char, param_help: *const c_char) -> Error,
}

#[derive(WrapperApi)]
pub struct CoreTasApi {
    #[dlopen2_name = "CoreTAS_SetInputFilterCallback"]
    set_input_callback:
        unsafe extern "C" fn(context: *mut c_void, callback: InputFilterCallback) -> Error,
    #[dlopen2_name = "CoreTAS_SetAudioCallbacks"]
    set_audio_callbacks: unsafe extern "C" fn(
        context: *mut c_void,
        rate_callback: AudioRateCallbck,
        sample_callback: AudioSampleCallback,
    ),
    #[dlopen2_name = "CoreTAS_SetAudioTapEnabled"]
    set_audio_tap_enabled: unsafe extern "C" fn(value: bool) -> Error,
}

#[derive(WrapperApi)]
pub struct BasePluginApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut PluginType,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> Error,
    #[dlopen2_name = "PluginStartup"]
    startup: unsafe extern "C" fn(
        core_lib_handle: DynlibHandle,
        debug_context: *mut c_void,
        debug_callback: unsafe extern "C" fn(
            context: *mut c_void,
            level: c_int,
            message: *const c_char,
        ),
    ) -> Error,
    #[dlopen2_name = "PluginShutdown"]
    shutdown: unsafe extern "C" fn() -> Error,
}
