use std::ffi::{c_int, c_uint, c_char, c_void};
use dlopen2::wrapper::{WrapperApi, WrapperMultiApi};

mod types;

pub use types::*;

#[derive(WrapperMultiApi)]
pub struct FullCoreApi {
    pub core: CoreBaseApi,
    pub vcr: CoreVcrApi
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
    attach_plugin: unsafe extern "C" fn(
        plugin_type: PluginType,
        plugin_lib_handle: DynlibHandle,
    ) -> Error,
    #[dlopen2_name = "CoreDetachPlugin"]
    detach_plugin: unsafe extern "C" fn(plugin_type: PluginType) -> Error,
    #[dlopen2_name = "CoreDoCommand"]
    do_command: unsafe extern "C" fn(
        command: Command,
        int_param: c_int,
        ptr_param: *mut c_void,
    ) -> Error,
    #[dlopen2_name = "CoreOverrideVidExt"]
    override_vidext: unsafe extern "C" fn(
        video_function_struct: *mut VideoExtensionFunctions,
    ) -> Error,
}
#[derive(WrapperApi)]
pub struct CoreVcrApi {
    #[dlopen2_name = "VCR_SetErrorCallback"]
    set_error_callback: unsafe extern "C" fn(
        callb: unsafe extern "C" fn(
            lvl: MsgLevel,
            msg: *const c_char
        )
    ),
    #[dlopen2_name = "VCR_SetStateCallback"]
    set_state_callback: unsafe extern "C" fn(
        callb: unsafe extern "C" fn(
            param: VcrParam,
            value: c_int
        )
    ),
    #[dlopen2_name = "VCR_GetCurFrame"]
    get_cur_frame: unsafe extern "C" fn() -> c_uint,

    #[dlopen2_name = "VCR_StopMovie"]
    stop_movie: unsafe extern "C" fn(restart: c_int),

    #[dlopen2_name = "VCR_SetOverlay"]
    set_overlay: unsafe extern "C" fn(keys: Buttons, channel: c_uint),

    #[dlopen2_name = "VCR_GetKeys"]
    get_keys: unsafe extern "C" fn(keys: *mut Buttons, channel: c_uint) -> c_int,

    #[dlopen2_name = "VCR_IsPlaying"]
    is_playing: unsafe extern "C" fn() -> c_int,

    #[dlopen2_name = "VCR_AdvanceFrame"]
    advance_frame: unsafe extern "C" fn() -> c_int,

    #[dlopen2_name = "VCR_ResetOverlay"]
    reset_overlay: unsafe extern "C" fn(),

    #[dlopen2_name = "VCR_IsReadOnly"]
    is_read_only: unsafe extern "C" fn() -> c_int,

    #[dlopen2_name = "VCR_SetReadOnly"]
    set_read_only: unsafe extern "C" fn(read_only: c_int) -> c_int,

    #[dlopen2_name = "VCR_StartRecording"]
    start_recording: unsafe extern "C" fn(
        path: *const c_char,
        author: *const c_char,
        description: *const c_char,
        start_type: VcrStartType
    ) -> Error,

    #[dlopen2_name = "VCR_StartMovie"]
    start_movie: unsafe extern "C" fn(
        path: *const c_char,
    ) -> Error
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
