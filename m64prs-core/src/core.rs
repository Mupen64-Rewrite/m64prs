use std::{
    ffi::{c_char, c_int, c_void, CStr}, fs, ops::{Deref, DerefMut}, path::Path, ptr::{null, null_mut}, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}
};

use dlopen2::wrapper::{Container, WrapperApi};

use crate::{
    ctypes::{
        m64p_command, m64p_core_param, m64p_dynlib_handle, m64p_error, m64p_msg_level, m64p_plugin_type, M64CMD_EXECUTE, M64CMD_ROM_CLOSE, M64CMD_ROM_OPEN, M64ERR_SUCCESS
    },
    enums::PluginType,
    error::{CoreError, M64PError, Result},
};

macro_rules! try_core_error {
    ($call:expr) => {
        unsafe {
            let retv = $call;
            if retv != M64ERR_SUCCESS {
                return Err(CoreError::M64P(retv.try_into().unwrap()));
            }
        }
    };
}

pub struct Core {
    lib: Container<CoreApi>,

    rsp_plugin: Option<Container<BasePluginApi>>,
    video_plugin: Option<Container<BasePluginApi>>,
    audio_plugin: Option<Container<BasePluginApi>>,
    input_plugin: Option<Container<BasePluginApi>>,
}
static CORE_SINGLETON: RwLock<Option<Core>> = RwLock::new(None);

impl Core {
    /// Loads the Mupen64Plus core from a path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<()> {
        {
            let mut core = CORE_SINGLETON.write().unwrap();
            if core.is_some() {
                drop(core);
                panic!("Core is already loaded!!");
            }

            *core = Some(Core {
                lib: unsafe { Container::load(path.as_ref()).map_err(|err| CoreError::Library(err))? },
                rsp_plugin: None,
                video_plugin: None,
                audio_plugin: None,
                input_plugin: None,
            });
        }

        unsafe extern "C" fn debug_callback(
            ctx: *mut c_void,
            level: c_int,
            message: *const c_char,
        ) {
            println!("DEBUG: {} {}", level, CStr::from_ptr(message).to_str().unwrap());
        }
        unsafe extern "C" fn state_callback(
            ctx: *mut c_void,
            param: m64p_core_param,
            value: c_int,
        ) {
            eprintln!("STATE: {} {}", param, value);
        }

        {
            let core = Self::get();
            unsafe {
                let retv = core.lib.startup(
                    0x02_01_00,
                    null(),
                    null(),
                    null_mut(),
                    debug_callback,
                    null_mut(),
                    state_callback,
                );
                if retv != M64ERR_SUCCESS {
                    let msg = CStr::from_ptr(core.lib.error_message(retv)).to_str();
                    panic!("Core failed to start up! ({})", msg.unwrap_or("Unknown error"));
                }
            }
        }

        Ok(())
    }

    pub fn get() -> CoreReadLock {
        let core = CORE_SINGLETON.read().unwrap();
        if core.is_none() {
            drop(core);
            panic!("Core isn't loaded!");
        }
        CoreReadLock { read_guard: core }
    }

    pub fn get_mut() -> CoreWriteLock {
        let core = CORE_SINGLETON.write().unwrap();
        if core.is_none() {
            drop(core);
            panic!("Core isn't loaded!");
        }
        CoreWriteLock { write_guard: core }
    }

    pub fn get_error_message(&self, code: M64PError) -> &'static str {
        unsafe {
            let ptr = self.lib.error_message(code as m64p_error);
            CStr::from_ptr(ptr).to_str().unwrap()
        }
    }

    /// Loads and attaches a plugin from a path. Checks that the plugin is actually the right type.
    /// Mupen64Plus requires that plugins be loaded in the order: video, audio, input, RSP.
    /// 
    /// # Errors
    /// TODO: document errors
    pub fn attach_plugin<P: AsRef<Path>>(
        &mut self,
        plugin_type: PluginType,
        path: P,
    ) -> Result<()> {
        let cur_plugin: &mut Option<Container<BasePluginApi>> = match plugin_type {
            PluginType::RSP => &mut self.rsp_plugin,
            PluginType::Graphics => &mut self.video_plugin,
            PluginType::Audio => &mut self.audio_plugin,
            PluginType::Input => &mut self.input_plugin,
        };
        if cur_plugin.is_some() {
            panic!("Plugin already attached for {}", plugin_type);
        }

        unsafe extern "C" fn debug_callback(
            ctx: *mut c_void,
            level: c_int,
            message: *const c_char,
        ) {
            println!("DEBUG: {} {}", level, CStr::from_ptr(message).to_str().unwrap());
        }

        let api = unsafe {
            Container::<BasePluginApi>::load(path.as_ref())
                .map_err(|err| CoreError::Library(err))?
        };
        {
            let mut reported_ptype: m64p_plugin_type = 0;

            // check the self-reported plugin type to verify
            try_core_error!(api.get_version(
                &mut reported_ptype,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            ));

            if reported_ptype != plugin_type as m64p_plugin_type {
                return Err(CoreError::PluginTypeNotMatching);
            }

            // startup the plugin
            try_core_error!(api.startup(
                self.lib.into_raw(), 
                null_mut(), 
                debug_callback
            ));

            // attach the plugin
            try_core_error!(self.lib.attach_plugin(plugin_type as m64p_plugin_type, api.into_raw()));
        }
        *cur_plugin = Some(api);

        Ok(())
    }

    pub fn detach_plugin(&mut self, plugin_type: PluginType) -> Result<()> {
        let cur_plugin: &mut Option<Container<BasePluginApi>> = match plugin_type {
            PluginType::RSP => &mut self.rsp_plugin,
            PluginType::Graphics => &mut self.video_plugin,
            PluginType::Audio => &mut self.audio_plugin,
            PluginType::Input => &mut self.input_plugin,
        };
        if cur_plugin.is_none() {
            panic!("No plugin attached for {}", plugin_type);
        }

        try_core_error!(self.lib.detach_plugin(plugin_type as m64p_plugin_type));

        *cur_plugin = None;
        Ok(())
    }

    /// Opens a ROM loaded as a byte array.
    pub fn open_rom(&mut self, data: &[u8]) -> Result<()> {
        try_core_error!(self.lib.do_command(M64CMD_ROM_OPEN, data.len() as c_int, data.as_ptr() as *mut c_void));
        Ok(())
    }

    /// Loads a ROM from a file and opens it.
    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let data = fs::read(path).map_err(|err| CoreError::IO(err))?;
        try_core_error!(self.lib.do_command(M64CMD_ROM_OPEN, data.len() as c_int, data.as_ptr() as *mut c_void));
        Ok(())
    }

    /// Closes a ROM.
    pub fn close_rom(&mut self) -> Result<()> {
        try_core_error!(self.lib.do_command(M64CMD_ROM_CLOSE, 0, null_mut()));
        Ok(())
    }

    /// Executes the ROM on this thread.
    pub fn execute_sync(&self) -> Result<()> {
        try_core_error!(self.lib.do_command(M64CMD_EXECUTE, 0, null_mut()));
        Ok(())
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe { self.lib.shutdown() };
    }
}

/// Read-only reference to the active Mupen64Plus library.
pub struct CoreReadLock {
    read_guard: RwLockReadGuard<'static, Option<Core>>,
}

impl Deref for CoreReadLock {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        self.read_guard.as_ref().unwrap()
    }
}

/// Read-write reference to the active Mupen64Plus library.
pub struct CoreWriteLock {
    write_guard: RwLockWriteGuard<'static, Option<Core>>,
}

impl Deref for CoreWriteLock {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        self.write_guard.as_ref().unwrap()
    }
}

impl DerefMut for CoreWriteLock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.write_guard.as_mut().unwrap()
    }
}

#[derive(WrapperApi)]
#[rustfmt::skip]
struct CoreApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut m64p_plugin_type,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int
    ) -> m64p_error,
    #[dlopen2_name = "CoreErrorMessage"]
    error_message: unsafe extern "C" fn(
        return_code: m64p_error
    ) -> *const c_char,
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
            new_value: c_int
        ),
    ) -> m64p_error,
    #[dlopen2_name = "CoreShutdown"]
    shutdown: unsafe extern "C" fn() -> m64p_error,
    #[dlopen2_name = "CoreAttachPlugin"]
    attach_plugin: unsafe extern "C" fn(
        plugin_type: m64p_plugin_type,
        plugin_lib_handle: m64p_dynlib_handle
    ) -> m64p_error,
    #[dlopen2_name = "CoreDetachPlugin"]
    detach_plugin: unsafe extern "C" fn(
        plugin_type: m64p_plugin_type
    ) -> m64p_error,
    #[dlopen2_name = "CoreDoCommand"]
    do_command: unsafe extern "C" fn(
        command: m64p_command,
        int_param: c_int,
        ptr_param: *mut c_void
    ) -> m64p_error,
}

#[derive(WrapperApi)]
#[rustfmt::skip]
struct BasePluginApi {
    #[dlopen2_name = "PluginGetVersion"]
    get_version: unsafe extern "C" fn(
        plugin_type: *mut m64p_plugin_type,
        plugin_version: *mut c_int,
        api_version: *mut c_int,
        plugin_name_ptr: *mut *const c_char,
        capabilities: *mut c_int
    ) -> m64p_error,
    #[dlopen2_name = "PluginStartup"]
    startup: unsafe extern "C" fn(
        core_lib_handle: m64p_dynlib_handle,
        debug_context: *mut c_void,
        debug_callback: unsafe extern "C" fn(
            context: *mut c_void,
            level: c_int,
            message: *const c_char,
        )
    ) -> m64p_error,
    #[dlopen2_name = "PluginShutdown"]
    shutdown: unsafe extern "C" fn() -> m64p_error
}
