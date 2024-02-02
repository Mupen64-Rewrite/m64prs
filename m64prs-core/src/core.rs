use std::{
    ffi::{c_char, c_int, c_void, CStr},
    fs,
    path::Path,
    ptr::{null, null_mut},
};

use dlopen2::wrapper::{Container, WrapperApi};

use crate::{
    ctypes::{
        m64p_command, m64p_core_param, m64p_dynlib_handle, m64p_error, m64p_plugin_type, M64CMD_EXECUTE, M64CMD_ROM_CLOSE, M64CMD_ROM_OPEN, M64ERR_SUCCESS
    },
    enums::{APIType, APIVersion, PluginType},
    error::{CoreError, Result},
};

fn try_m64p_error(return_code: m64p_error) -> Result<()> {
    match return_code {
        M64ERR_SUCCESS => Ok(()),
        error => Err(CoreError::M64P(error.try_into().unwrap())),
    }
}

#[allow(unused)]
extern "C" fn debug_callback(context: *mut c_void, level: c_int, message: *const c_char) {}

#[allow(unused)]
extern "C" fn state_callback(context: *mut c_void, param: m64p_core_param, new_value: c_int) {}

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

        try_m64p_error(unsafe {
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

    /// Attaches a plugin (`plugin`) of type `ptype`. Panics if there is already a plugin attached for that type.
    pub fn attach_plugin(&mut self, ptype: PluginType, plugin: Plugin) -> Result<()> {
        let cur_plugin = match ptype {
            PluginType::RSP => &mut self.rsp_plugin,
            PluginType::Video => &mut self.video_plugin,
            PluginType::Audio => &mut self.audio_plugin,
            PluginType::Input => &mut self.input_plugin,
        };

        // make sure we don't have a plugin attached already
        if cur_plugin.is_some() {
            panic!("There is already a {} plugin attached", ptype);
        }

        // check that the plugin is in fact the right kind
        if let Ok(ver) = plugin.get_version() {
            match ver.api_type {
                APIType::Core => return Err(CoreError::PluginTypeNotMatching),
                APIType::Plugin(plugin_ptype) => {
                    if plugin_ptype != ptype {
                        return Err(CoreError::PluginTypeNotMatching);
                    }
                }
            }
        }

        // startup and attach the plugin
        try_m64p_error(unsafe {
            plugin
                .lib
                .startup(self.lib.into_raw(), null_mut(), debug_callback)
        })?;
        try_m64p_error(unsafe { self.lib.attach_plugin(ptype.into(), plugin.lib.into_raw()) })?;

        // register the plugin with the struct
        *cur_plugin = Some(plugin);

        Ok(())
    }

    /// Detaches a plugin by type.
    pub fn detach_plugin(&mut self, ptype: PluginType) -> Result<()> {
        let cur_plugin = match ptype {
            PluginType::RSP => &mut self.rsp_plugin,
            PluginType::Video => &mut self.video_plugin,
            PluginType::Audio => &mut self.audio_plugin,
            PluginType::Input => &mut self.input_plugin,
        };

        // detach the plugin.
        try_m64p_error(unsafe { self.lib.detach_plugin(ptype.into()) })?;

        // assign the plugin to none, this should drop it.
        *cur_plugin = None;

        Ok(())
    }

    pub fn open_rom(&mut self, rom_data: &[u8]) -> Result<()> {
        try_m64p_error(unsafe {
            self.lib.do_command(
                M64CMD_ROM_OPEN,
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
        try_m64p_error(unsafe { self.lib.do_command(M64CMD_ROM_CLOSE, 0, null_mut()) })?;
        Ok(())
    }

    pub fn execute_sync(&self) -> Result<()> {
        try_m64p_error(unsafe { self.lib.do_command(M64CMD_EXECUTE, 0, null_mut()) })?;
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
            let mut plugin_type: m64p_plugin_type = 0;
            let mut plugin_version: c_int = 0;
            let mut api_version: c_int = 0;
            let mut plugin_name: *const c_char = null();

            try_m64p_error(self.lib.get_version(
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
        plugin_type: *mut m64p_plugin_type,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> m64p_error,
    #[dlopen2_name = "CoreErrorMessage"]
    error_message: unsafe extern "C" fn(return_code: m64p_error) -> *const c_char,
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
            param: m64p_core_param,
            new_value: c_int,
        ),
    ) -> m64p_error,
    #[dlopen2_name = "CoreShutdown"]
    shutdown: unsafe extern "C" fn() -> m64p_error,
    #[dlopen2_name = "CoreAttachPlugin"]
    attach_plugin: unsafe extern "C" fn(
        plugin_type: m64p_plugin_type,
        plugin_lib_handle: m64p_dynlib_handle,
    ) -> m64p_error,
    #[dlopen2_name = "CoreDetachPlugin"]
    detach_plugin: unsafe extern "C" fn(plugin_type: m64p_plugin_type) -> m64p_error,
    #[dlopen2_name = "CoreDoCommand"]
    do_command: unsafe extern "C" fn(
        command: m64p_command,
        int_param: c_int,
        ptr_param: *mut c_void,
    ) -> m64p_error,
}

#[derive(WrapperApi)]
struct BasePluginApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut m64p_plugin_type,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int,
    ) -> m64p_error,
    #[dlopen2_name = "PluginStartup"]
    startup: unsafe extern "C" fn(
        core_lib_handle: m64p_dynlib_handle,
        debug_context: *mut c_void,
        debug_callback: unsafe extern "C" fn(
            context: *mut c_void,
            level: c_int,
            message: *const c_char,
        ),
    ) -> m64p_error,
    #[dlopen2_name = "PluginShutdown"]
    shutdown: unsafe extern "C" fn() -> m64p_error,
}
