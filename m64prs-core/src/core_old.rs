use std::{
    ffi::{c_char, c_int, c_uint, c_void, CStr, CString},
    fs, mem,
    path::Path,
    ptr::{null, null_mut},
    slice, sync::{atomic::AtomicBool, Arc, OnceLock, RwLock},
};

use crate::{
    ctypes::{self, Command, GLAttribute, MsgLevel, PluginType, Size2D, VcrParam, VideoFlags, VideoMode},
    error::{CoreError, Result},
    types::{APIVersion, VideoExtension},
};
use ash::vk::{self, Handle};
use dlopen2::wrapper::{Container, WrapperApi, WrapperMultiApi};

use log::{log, Level};
use normalize_path::NormalizePath;

mod api;

fn test_c_err(return_code: ctypes::Error) -> Result<()> {
    match return_code {
        ctypes::Error::SUCCESS => Ok(()),
        error => Err(CoreError::M64P(error.try_into().unwrap())),
    }
}

/// Macro used to avoid some boilerplate for writing generic wrapper stubs for video extension.
/// Syntax is similar to function syntax, except minus the "fn name" at the beginning.
macro_rules! vidext_fn_wrapper {
    (( $($pname:ident : $ptype:ty),* ) -> $rtype:ty { $($code:tt)* }) => {
        {
            unsafe extern "C" fn f<V: VideoExtension>($($pname: $ptype),*) -> $rtype {
                $($code)*
            }
            Some(f::<V>)
        }
    };

    (( $($pname:ident : $ptype:ty),* ) { $($code:tt)* }) => {
        {
            unsafe extern "C" fn f<V: VideoExtension>($($pname: $ptype),*) {
                $($code)*
            }
            Some(f::<V>)
        }
    };
}

/// Macro used to replace some FFI boilerplate (bridging the Rust video extension to C).
/// This returns an error as a C error code if needed, returning the result.
macro_rules! match_ffi_result {
    ($e:expr) => {
        match $e {
            Ok(res) => res,
            Err(err) => return err.into(),
        }
    };
}

#[allow(unused)]
unsafe extern "C" fn debug_callback(context: *mut c_void, level: c_int, message: *const c_char) {
    let log_level = match MsgLevel(level as c_uint) {
        MsgLevel::ERROR => Level::Error,
        MsgLevel::WARNING => Level::Warn,
        MsgLevel::INFO => Level::Info,
        MsgLevel::STATUS => Level::Debug,
        MsgLevel::VERBOSE => Level::Trace,
        _ => panic!("Received invalid message level {}", level),
    };
    log!(log_level, "{}", CStr::from_ptr(message).to_str().unwrap());
}

#[allow(unused)]
extern "C" fn state_callback(context: *mut c_void, param: ctypes::CoreParam, new_value: c_int) {
    let core = unsafe { &*(context as *const Core) };
    core.notify_state_change(param, new_value);
}

#[derive(Clone, Copy, Debug)]
pub struct StateEvent {
    pub param: ctypes::CoreParam,
    pub new_value: c_int,
}

/// Holds a loaded instance of the Mupen64Plus core. Note that the core remains loaded and active for the lifetime of this
/// struct. It is shut down and unloaded when the struct is dropped.
pub struct Core {
    lib: Container<FullCoreApi>,

    rsp_plugin: Option<Plugin>,
    video_plugin: Option<Plugin>,
    audio_plugin: Option<Plugin>,
    input_plugin: Option<Plugin>,
}

static CORE_INSTANCE: OnceLock<RwLock<Core>> = OnceLock::new();

//
impl Core {
    /// Loads the core from a path. Panics if library loading or initialization fails.
    pub fn load<P: AsRef<Path>>(path: P) -> &'static RwLock<Self> {
        let lock = CORE_INSTANCE.get_or_init(|| {
            let core = Core {
                lib: unsafe { Container::load(path.as_ref()) }.unwrap(),
                rsp_plugin: None,
                video_plugin: None,
                audio_plugin: None,
                input_plugin: None,
            };

            test_c_err(unsafe {
                core.lib.core.startup(
                    0x02_01_00,
                    null(),
                    null(),
                    null_mut(),
                    debug_callback,
                    (&core as *const Core) as *mut c_void,
                    state_callback,
                )
            }).unwrap();

            RwLock::new(core)
        });
        lock
    }

    pub fn get_lock() -> &'static RwLock<Self> {
        CORE_INSTANCE.get().expect("Core is not initialized.")
    }

    fn notify_state_change(&self, param: ctypes::CoreParam, value: c_int) {
        let _ = (param, value);
    }

    /// Obtains version information about this core.
    pub fn get_version(&self) -> Result<APIVersion> {
        unsafe {
            let mut plugin_type: ctypes::PluginType = ctypes::PluginType::NULL;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();
            let mut caps: c_int = 0;

            test_c_err(self.lib.core.get_version(
                &mut plugin_type,
                &mut plugin_version,
                &mut api_version,
                &mut plugin_name,
                &mut caps,
            ))?;

            Ok(APIVersion {
                api_type: plugin_type.try_into().unwrap(),
                plugin_version: plugin_version,
                api_version: api_version,
                plugin_name: CStr::from_ptr(plugin_name).to_str().unwrap(),
                capabilities: caps,
            })
        }
    }

    /// Attaches a plugin (`plugin`) of type `ptype`. Panics if there is already a plugin attached for that type.
    pub fn attach_plugin(&mut self, ptype: PluginType, plugin: Plugin) -> Result<()> {
        let cur_plugin = match ptype {
            PluginType::RSP => &mut self.rsp_plugin,
            PluginType::GFX => &mut self.video_plugin,
            PluginType::AUDIO => &mut self.audio_plugin,
            PluginType::INPUT => &mut self.input_plugin,
            _ => panic!("Invalid plugin type!"),
        };

        // make sure we don't have a plugin attached already
        if cur_plugin.is_some() {
            panic!("There is already a {:?} plugin attached", ptype);
        }

        // check that the plugin is in fact the right kind
        if let Ok(ver) = plugin.get_version() {
            if ver.api_type != ptype {
                return Err(CoreError::MismatchedPluginType);
            }
        }

        // startup and attach the plugin
        test_c_err(unsafe {
            plugin
                .lib
                .startup(self.lib.into_raw(), null_mut(), debug_callback)
        })?;
        test_c_err(unsafe { self.lib.core.attach_plugin(ptype.into(), plugin.lib.into_raw()) })?;

        // register the plugin with the struct
        *cur_plugin = Some(plugin);

        Ok(())
    }

    /// Detaches a plugin by type.
    pub fn detach_plugin(&mut self, ptype: PluginType) -> Result<()> {
        let cur_plugin = match ptype {
            PluginType::RSP => &mut self.rsp_plugin,
            PluginType::GFX => &mut self.video_plugin,
            PluginType::AUDIO => &mut self.audio_plugin,
            PluginType::INPUT => &mut self.input_plugin,
            _ => panic!("Invalid plugin type!"),
        };

        // detach the plugin.
        test_c_err(unsafe { self.lib.core.detach_plugin(ptype.into()) })?;

        // assign the plugin to none, this should drop it.
        *cur_plugin = None;

        Ok(())
    }
    fn override_vidext_inner<V: VideoExtension>(&mut self) -> Result<()> {
        // generate wrapper functions with a helper macro
        let mut vidext = ctypes::VideoExtensionFunctions {
            Functions: 17,
            VidExtFuncInit: vidext_fn_wrapper!(() -> ctypes::Error {
                match_ffi_result!(V::init());
                ctypes::Error::SUCCESS
            }),
            VidExtFuncQuit: vidext_fn_wrapper!(() -> ctypes::Error {
                match_ffi_result!(V::quit());
                ctypes::Error::SUCCESS
            }),
            VidExtFuncListModes: vidext_fn_wrapper!((size_array: *mut Size2D, num_sizes: *mut c_int) -> ctypes::Error {
                let size_in_src = match_ffi_result!(V::list_fullscreen_modes());
                let size_in = size_in_src.as_ref();
                let size_out = slice::from_raw_parts_mut(size_array, (*num_sizes).try_into().unwrap());

                if size_in.len() < size_out.len() {
                    size_out[..size_in.len()].copy_from_slice(size_in);
                    *num_sizes = size_in.len().try_into().unwrap();
                }
                else {
                    size_out.copy_from_slice(&size_in[..size_out.len()]);
                    *num_sizes = size_out.len().try_into().unwrap();
                }

                ctypes::Error::SUCCESS
            }),
            VidExtFuncListRates: vidext_fn_wrapper!((size: Size2D, num_rates: *mut c_int, rates: *mut c_int) -> ctypes::Error {
                let rate_in_src = match_ffi_result!(V::list_fullscreen_rates(size));
                let rate_in = rate_in_src.as_ref();
                let rate_out = slice::from_raw_parts_mut(rates, *num_rates as usize);

                if rate_in.len() < rate_out.len() {
                    rate_out[..rate_in.len()].copy_from_slice(rate_in);
                    *num_rates = rate_in.len().try_into().unwrap();
                }
                else {
                    rate_out.copy_from_slice(&rate_in[..rate_out.len()]);
                    *num_rates = rate_out.len().try_into().unwrap();
                }

                ctypes::Error::SUCCESS
            }),
            VidExtFuncSetMode: vidext_fn_wrapper!((
                width: c_int,
                height: c_int,
                bits_per_pixel: c_int,
                screen_mode: c_int,
                flags: c_int
            ) -> ctypes::Error {
                match_ffi_result!(V::set_video_mode(
                    width, height, bits_per_pixel, VideoMode(screen_mode as u32), VideoFlags(flags as u32)
                ));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncSetModeWithRate: vidext_fn_wrapper!((
                width: c_int,
                height: c_int,
                refresh_rate: c_int,
                bits_per_pixel: c_int,
                screen_mode: c_int,
                flags: c_int
            ) -> ctypes::Error {
                match_ffi_result!(V::set_video_mode_with_rate(
                    width, height, bits_per_pixel, refresh_rate, VideoMode(screen_mode as u32), VideoFlags(flags as u32)
                ));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncGLGetProc: vidext_fn_wrapper!((proc: *const c_char) -> ctypes::Function {
                // they want a function pointer, not a wimpy normal pointer
                mem::transmute(V::gl_get_proc_address(CStr::from_ptr(proc)))
            }),
            VidExtFuncGLSetAttr: vidext_fn_wrapper!((attr: GLAttribute, value: c_int) -> ctypes::Error {
                match_ffi_result!(V::gl_set_attribute(attr, value));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncGLGetAttr: vidext_fn_wrapper!((attr: GLAttribute, value: *mut c_int) -> ctypes::Error {
                *value = match_ffi_result!(V::gl_get_attribute(attr));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncGLSwapBuf: vidext_fn_wrapper!(() -> ctypes::Error {
                match_ffi_result!(V::gl_swap_buffers());
                ctypes::Error::SUCCESS
            }),
            VidExtFuncSetCaption: vidext_fn_wrapper!((title: *const c_char) -> ctypes::Error {
                match_ffi_result!(V::set_caption(CStr::from_ptr(title)));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncToggleFS: vidext_fn_wrapper!(() -> ctypes::Error {
                match_ffi_result!(V::toggle_full_screen());
                ctypes::Error::SUCCESS
            }),
            VidExtFuncResizeWindow: vidext_fn_wrapper!((width: c_int, height: c_int) -> ctypes::Error {
                match_ffi_result!(V::resize_window(width, height));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncGLGetDefaultFramebuffer: vidext_fn_wrapper!(() -> u32 {
                V::gl_get_default_framebuffer()
            }),
            VidExtFuncInitWithRenderMode: vidext_fn_wrapper!((mode: ctypes::RenderMode) -> ctypes::Error {
                match_ffi_result!(V::init_with_render_mode(mode));
                ctypes::Error::SUCCESS
            }),
            VidExtFuncVKGetSurface: vidext_fn_wrapper!((surface: *mut *mut c_void, instance: *mut c_void) -> ctypes::Error {
                *surface = match_ffi_result!(
                    V::vk_get_surface(vk::Instance::from_raw(instance as isize as u64))
                ).as_raw() as *mut c_void;

                ctypes::Error::SUCCESS
            }),
            VidExtFuncVKGetInstanceExtensions: vidext_fn_wrapper!((extensions: *mut *mut *const c_char, count: *mut u32) -> ctypes::Error {
                let data = match_ffi_result!(V::vk_get_instance_extensions());
                
                // This is unsafe, though Mupen shouldn't be mutating it, so we're fine.
                *extensions = data.as_ptr() as *mut *const i8;
                *count = data.len().try_into().unwrap();

                ctypes::Error::SUCCESS
            }),
        };
        // pass wrapper functions to Mupen
        test_c_err(unsafe { self.lib.core.override_vidext(&mut vidext) })?;
        Ok(())
    }

    /// Overrides the video extension.
    pub fn override_vidext<V: VideoExtension>(this: &Arc<RwLock<Self>>) -> Result<()> {
        V::on_bind_core(this.clone()).map_err(|err| CoreError::M64P(err))?;
        this.write().unwrap().override_vidext_inner::<V>()?;

        Ok(())
    }
}

// Core commands and state
impl Core {
    /// Opens a ROM that is pre-loaded into memory.
    pub fn open_rom(&self, rom_data: &[u8]) -> Result<()> {
        test_c_err(unsafe {
            self.lib.core.do_command(
                Command::ROM_OPEN,
                rom_data.len() as c_int,
                rom_data.as_ptr() as *mut c_void,
            )
        })?;

        Ok(())
    }

    /// Loads and opens a ROM from a given file path.
    pub fn load_rom<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let rom_data = fs::read(path.as_ref()).map_err(|err| CoreError::IO(err))?;
        self.open_rom(&rom_data)
    }

    /// Closes a currently open ROM.
    pub fn close_rom(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::ROM_CLOSE, 0, null_mut()) })
    }

    /// Executes the currently open ROM synchronously on this thread.
    pub fn execute_sync(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::EXECUTE, 0, null_mut()) })
    }

    /// Stops the emulator if it is running.    
    pub fn stop(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::STOP, 0, null_mut()) })
    }

    /// Pauses the emulator if it is running.
    pub fn pause(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::PAUSE, 0, null_mut()) })
    }

    /// Resumes the emulator if it is paused.
    pub fn resume(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::RESUME, 0, null_mut()) })
    }

    /// Advances one frame. That is, the emulator will run the next frame, then pause.
    pub fn advance_frame(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::ADVANCE_FRAME, 0, null_mut()) })
    }

    /// Resets the emulator. If the `hard` parameter is true, performs a hard reset as opposed to a soft reset.
    pub fn reset(&self, hard: bool) -> Result<()> {
        test_c_err(unsafe {
            self.lib.core
                .do_command(Command::RESUME, if hard { 1 } else { 0 }, null_mut())
        })
    }

    /// Sets the current savestate slot. The savestate slot must be a value from 0 to 9.
    pub fn set_state_slot(&self, index: u32) -> Result<()> {
        if index >= 10 {
            panic!(
                "Invalid savestate index {} (savestate indices are between 0 and 9 inclusive)",
                index
            );
        }
        test_c_err(unsafe {
            self.lib.core
                .do_command(Command::STATE_SET_SLOT, index as c_int, null_mut())
        })
    }

    /// Saves to the current savestate slot. This function returns immediately, use (TODO: do this) to be notified when it's finished.
    pub fn save_current_slot(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::STATE_SAVE, 0, null_mut()) })
    }

    /// Saves to a file. This function returns immediately, use (TODO: do this) to be notified when it's finished.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let canon_path = path.as_ref().normalize();
        let path_cstr = CString::new(canon_path.to_string_lossy().as_bytes()).unwrap();

        test_c_err(unsafe {
            self.lib.core
                .do_command(Command::STATE_SAVE, 0, path_cstr.as_ptr() as *mut c_void)
        })
    }

    /// Loads from the current savestate slot. This function returns immediately, use (TODO: do this) to be notified when it's finished.
    pub fn load_current_slot(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.core.do_command(Command::STATE_LOAD, 0, null_mut()) })
    }

    /// Loads from a file. This function returns immediately, use (TODO: do this) to be notified when it's finished.
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let canon_path = path
            .as_ref()
            .canonicalize()
            .map_err(|err| CoreError::IO(err))?;
        let path_cstr = CString::new(canon_path.to_string_lossy().as_bytes()).unwrap();

        test_c_err(unsafe {
            self.lib.core
                .do_command(Command::STATE_LOAD, 0, path_cstr.as_ptr() as *mut c_void)
        })
    }
}

// VCR functions
impl Core {

}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe { self.lib.core.shutdown() };
    }
}

/// Holds a loaded instance of a Mupen64Plus plugin. The core is responsible for startup/shutdown of
/// plugins, so plugins will remain unstarted when you have access to them.
pub struct Plugin {
    lib: Container<BasePluginApi>,
}

impl Plugin {
    /// Loads a plugin from a path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let plugin = Plugin {
            lib: unsafe { Container::load(path.as_ref()) }
                .map_err(|err| CoreError::Library(err))?,
        };

        Ok(plugin)
    }

    /// Obtains version information about this plugin.
    pub fn get_version(&self) -> Result<APIVersion> {
        unsafe {
            let mut plugin_type: PluginType = PluginType::NULL;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();

            test_c_err(self.lib.get_version(
                &mut plugin_type,
                &mut plugin_version,
                &mut api_version,
                &mut plugin_name,
                null_mut(),
            ))?;

            Ok(APIVersion {
                api_type: plugin_type.try_into().unwrap(),
                plugin_version: plugin_version,
                api_version: api_version,
                plugin_name: CStr::from_ptr(plugin_name).to_str().unwrap(),
                capabilities: 0,
            })
        }
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        unsafe {
            self.lib.shutdown();
        }
    }
}

#[derive(WrapperMultiApi)]
struct FullCoreApi {
    core: CoreBaseApi,
    vcr: CoreVcrApi
}

#[derive(WrapperApi)]
struct CoreBaseApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut PluginType,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> ctypes::Error,
    #[dlopen2_name = "CoreErrorMessage"]
    error_message: unsafe extern "C" fn(return_code: ctypes::Error) -> *const c_char,
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
            param: ctypes::CoreParam,
            new_value: c_int,
        ),
    ) -> ctypes::Error,
    #[dlopen2_name = "CoreShutdown"]
    shutdown: unsafe extern "C" fn() -> ctypes::Error,
    #[dlopen2_name = "CoreAttachPlugin"]
    attach_plugin: unsafe extern "C" fn(
        plugin_type: PluginType,
        plugin_lib_handle: ctypes::DynlibHandle,
    ) -> ctypes::Error,
    #[dlopen2_name = "CoreDetachPlugin"]
    detach_plugin: unsafe extern "C" fn(plugin_type: PluginType) -> ctypes::Error,
    #[dlopen2_name = "CoreDoCommand"]
    do_command: unsafe extern "C" fn(
        command: ctypes::Command,
        int_param: c_int,
        ptr_param: *mut c_void,
    ) -> ctypes::Error,
    #[dlopen2_name = "CoreOverrideVidExt"]
    override_vidext: unsafe extern "C" fn(
        video_function_struct: *mut ctypes::VideoExtensionFunctions,
    ) -> ctypes::Error,
}
#[derive(WrapperApi)]
struct CoreVcrApi {
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
    set_overlay: unsafe extern "C" fn(keys: ctypes::Buttons, channel: c_uint),

    #[dlopen2_name = "VCR_GetKeys"]
    get_keys: unsafe extern "C" fn(keys: *mut ctypes::Buttons, channel: c_uint) -> c_int,

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
        start_type: ctypes::VcrStartType
    ) -> ctypes::Error,

    #[dlopen2_name = "VCR_StartMovie"]
    start_movie: unsafe extern "C" fn(
        path: *const c_char,
    ) -> ctypes::Error
}

#[derive(WrapperApi)]
struct BasePluginApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut ctypes::PluginType,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> ctypes::Error,
    #[dlopen2_name = "PluginStartup"]
    startup: unsafe extern "C" fn(
        core_lib_handle: ctypes::DynlibHandle,
        debug_context: *mut c_void,
        debug_callback: unsafe extern "C" fn(
            context: *mut c_void,
            level: c_int,
            message: *const c_char,
        ),
    ) -> ctypes::Error,
    #[dlopen2_name = "PluginShutdown"]
    shutdown: unsafe extern "C" fn() -> ctypes::Error,
}
