use std::{
    ffi::{c_char, c_int, c_void, CStr},
    fs, mem,
    path::Path,
    ptr::{null, null_mut},
    slice,
};

use crate::{
    ctypes::{
        self, Command, GLAttribute, PluginType, Size2D, VideoExtensionFunctions, VideoFlags,
        VideoMode,
    },
    error::{CoreError, Result},
    types::{APIVersion, VideoExtension},
};
use dlopen2::wrapper::{Container, WrapperApi};

fn test_c_err(return_code: ctypes::Error) -> Result<()> {
    match return_code {
        ctypes::Error::SUCCESS => Ok(()),
        error => Err(CoreError::M64P(error.try_into().unwrap())),
    }
}

/// Macro used to avoid some boilerplate for writing generic wrapper stubs for video extension.
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

macro_rules! match_ffi_result {
    ($e:expr) => {
        match $e {
            Ok(res) => res,
            Err(err) => return err.into(),
        }
    };
}

#[allow(unused)]
extern "C" fn debug_callback(context: *mut c_void, level: c_int, message: *const c_char) {}

#[allow(unused)]
extern "C" fn state_callback(context: *mut c_void, param: ctypes::CoreParam, new_value: c_int) {}

pub struct Core {
    lib: Container<CoreApi>,

    rsp_plugin: Option<Plugin>,
    video_plugin: Option<Plugin>,
    audio_plugin: Option<Plugin>,
    input_plugin: Option<Plugin>,
}

impl Core {
    /// Loads the core from a path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let core = Core {
            lib: unsafe { Container::load(path.as_ref()) }
                .map_err(|err| CoreError::Library(err))?,
            rsp_plugin: None,
            video_plugin: None,
            audio_plugin: None,
            input_plugin: None,
        };

        test_c_err(unsafe {
            core.lib.startup(
                0x02_01_00,
                null(),
                null(),
                null_mut(),
                debug_callback,
                null_mut(),
                state_callback,
            )
        })?;

        Ok(core)
    }

    pub fn get_version(&self) -> Result<APIVersion> {
        unsafe {
            let mut plugin_type: ctypes::PluginType = ctypes::PluginType::NULL;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();
            let mut caps: c_int = 0;

            test_c_err(self.lib.get_version(
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
        test_c_err(unsafe { self.lib.attach_plugin(ptype.into(), plugin.lib.into_raw()) })?;

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
        test_c_err(unsafe { self.lib.detach_plugin(ptype.into()) })?;

        // assign the plugin to none, this should drop it.
        *cur_plugin = None;

        Ok(())
    }

    pub fn override_vidext<V: VideoExtension>(&mut self) -> Result<()> {
        // generate wrapper functions with a helper macro
        let mut vidext = ctypes::VideoExtensionFunctions {
            Functions: 14,
            VidExtFuncInit: vidext_fn_wrapper!(() -> ctypes::Error {
                match_ffi_result!(V::init());
                ctypes::Error::SUCCESS
            }),
            VidExtFuncQuit: vidext_fn_wrapper!(() -> ctypes::Error {
                match_ffi_result!(V::quit());
                ctypes::Error::SUCCESS
            }),
            VidExtFuncListModes: vidext_fn_wrapper!((size_array: *mut Size2D, num_sizes: *mut c_int) -> ctypes::Error {
                let iter = match_ffi_result!(V::list_fullscreen_modes());
                let slice = slice::from_raw_parts_mut(size_array, *num_sizes as usize);

                let mut count: c_int = 0;
                for (dst, src) in slice.iter_mut().zip(iter) {
                    count += 1;
                    *dst = src;
                }
                *num_sizes = count;

                ctypes::Error::SUCCESS
            }),
            VidExtFuncListRates: vidext_fn_wrapper!((size: Size2D, num_rates: *mut c_int, rates: *mut c_int) -> ctypes::Error {
                let iter = match_ffi_result!(V::list_fullscreen_rates(size));
                let slice = slice::from_raw_parts_mut(rates, *num_rates as usize);

                let mut count: c_int = 0;
                for (dst, src) in slice.iter_mut().zip(iter) {
                    count += 1;
                    *dst = src;
                }
                *num_rates = count;

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
        };
        // pass wrapper functions to Mupen
        test_c_err(unsafe { self.lib.override_vidext(&mut vidext) })?;
        Ok(())
    }

    pub fn open_rom(&mut self, rom_data: &[u8]) -> Result<()> {
        test_c_err(unsafe {
            self.lib.do_command(
                Command::ROM_OPEN,
                rom_data.len() as c_int,
                rom_data.as_ptr() as *mut c_void,
            )
        })?;

        Ok(())
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let rom_data = fs::read(path.as_ref()).map_err(|err| CoreError::IO(err))?;
        self.open_rom(&rom_data)
    }

    pub fn close_rom(&mut self) -> Result<()> {
        test_c_err(unsafe { self.lib.do_command(Command::ROM_CLOSE, 0, null_mut()) })?;
        Ok(())
    }

    pub fn execute_sync(&self) -> Result<()> {
        test_c_err(unsafe { self.lib.do_command(Command::EXECUTE, 0, null_mut()) })?;
        Ok(())
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe { self.lib.shutdown() };
    }
}

pub struct Plugin {
    lib: Container<BasePluginApi>,
}

impl Plugin {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let plugin = Plugin {
            lib: unsafe { Container::load(path.as_ref()) }
                .map_err(|err| CoreError::Library(err))?,
        };

        Ok(plugin)
    }

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

#[derive(WrapperApi)]
struct CoreApi {
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
    #[dlopen2_name = "CoreOverrideVidext"]
    override_vidext: unsafe extern "C" fn(
        video_function_struct: *mut ctypes::VideoExtensionFunctions,
    ) -> ctypes::Error,
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
