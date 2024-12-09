// CONFIGURATION API
// ===================

use std::{
    ffi::{c_char, c_float, c_int, c_void, CStr, CString},
    path::PathBuf,
    ptr::{null, null_mut},
};

use m64prs_sys::ConfigType;

use crate::error::{M64PError, WrongConfigType};

use super::{core_fn, Core};

/// Functions for the configuration system.
impl Core {
    pub fn cfg_shared_data_filepath(&self, name: &CStr) -> Option<PathBuf> {
        let path_ptr = unsafe { self.api.config.shared_data_filepath(name.as_ptr()) };
        if path_ptr.is_null() {
            None
        } else {
            // SAFETY: Mupen should return a valid pointer.
            Some(
                unsafe { CStr::from_ptr(path_ptr) }
                    .to_string_lossy()
                    .to_string()
                    .into(),
            )
        }
    }

    /// Runs the provided callback once per available config section.
    pub fn cfg_for_each_section<F: FnMut(&CStr)>(
        &mut self,
        mut callback: F,
    ) -> Result<(), M64PError> {
        unsafe extern "C" fn run_callback<F: FnMut(&CStr)>(
            context: *mut c_void,
            name: *const c_char,
        ) {
            let function: &mut F = &mut *(context as *mut F);
            function(CStr::from_ptr(name));
        }

        // SAFETY: the reference to the callback should only be used
        // during the function, it is not stored.
        core_fn(unsafe {
            self.api
                .config
                .list_sections(&mut callback as *mut F as *mut c_void, run_callback::<F>)
        })?;

        Ok(())
    }
    /// Opens the config section with the given name.
    pub fn cfg_open(&mut self, name: &CStr) -> Result<ConfigSection, M64PError> {
        let mut handle: m64prs_sys::Handle = null_mut();
        // SAFETY: the returned handle is guaranteed to be valid if the function
        // returns successfully.
        core_fn(unsafe {
            self.api
                .config
                .open_section(name.as_ptr(), &mut handle as *mut m64prs_sys::Handle)
        })?;

        // SAFETY: the lifetime of a ConfigSection cannot exceed that of the core it
        // references. This means that all functions should be callable during that time.
        Ok(ConfigSection {
            core: self,
            name: name.to_owned(),
            handle,
        })
    }

    pub fn cfg_save_all(&mut self) -> Result<(), M64PError> {
        core_fn(unsafe {
            self.api
                .config
                .save_file()
        })
    }
}
/// Represents a handle to a configuration section.
///
/// Each configuration section contains a list of parameters with values of variable type.
pub struct ConfigSection<'a> {
    core: &'a mut Core,
    name: CString,
    handle: m64prs_sys::Handle,
}

impl ConfigSection<'_> {
    /// Returns the section's name.
    pub fn name(&self) -> &CStr {
        &self.name
    }

    /// Saves the current section to disk.
    pub fn save(&mut self) -> Result<(), M64PError> {
        // SAFETY: the reference to the name should only be used
        // within the function, it is not stored.
        core_fn(unsafe { self.core.api.config.save_section(self.name.as_ptr()) })
    }

    /// Reverts any unsaved changes in this section.
    pub fn revert(&mut self) -> Result<(), M64PError> {
        // SAFETY: the reference to the name should only be used
        // within the function, it is not stored.
        core_fn(unsafe { self.core.api.config.revert_section(self.name.as_ptr()) })
    }

    /// Runs the provided callback once for each parameter in the section.
    /// The callback receives both the parameter's name and type.
    pub fn for_each_param<F: FnMut(&CStr, ConfigType)>(
        &self,
        mut callback: F,
    ) -> Result<(), M64PError> {
        unsafe extern "C" fn run_callback<F: FnMut(&CStr, ConfigType)>(
            context: *mut c_void,
            name: *const c_char,
            ptype: ConfigType,
        ) {
            let function: &mut F = &mut *(context as *mut F);
            function(CStr::from_ptr(name), ptype);
        }

        // SAFETY: the callback is only used within list_parameters,
        // it is not used after that.
        core_fn(unsafe {
            self.core.api.config.list_parameters(
                self.handle,
                &mut callback as *mut F as *mut c_void,
                run_callback::<F>,
            )
        })?;

        Ok(())
    }

    /// Gets the type of a parameter.
    pub fn get_type(&self, param: &CStr) -> Result<ConfigType, M64PError> {
        let mut param_type = ConfigType::Int;
        // SAFETY: the reference to the callback should only be used
        // during the function, it is not stored.
        core_fn(unsafe {
            self.core
                .api
                .config
                .get_parameter_type(self.handle, param.as_ptr(), &mut param_type)
        })?;

        Ok(param_type)
    }

    /// Gets the help string for a parameter.
    pub fn get_help(&self, param: &CStr) -> Result<CString, M64PError> {
        unsafe {
            // SAFETY: the CString passed here is only used within
            // the function.
            let help_ptr = self
                .core
                .api
                .config
                .get_parameter_help(self.handle, param.as_ptr());

            if help_ptr.is_null() {
                Err(M64PError::InputNotFound)
            } else {
                // SAFETY: the CString returned by Mupen should last
                // as long as it isn't overwritten.
                Ok(CStr::from_ptr(help_ptr).to_owned())
            }
        }
    }

    /// Gets the value of a parameter.
    pub fn get(&self, param: &CStr) -> Result<ConfigValue, M64PError> {
        let param_type = self.get_type(param)?;

        match param_type {
            ConfigType::Int => Ok(ConfigValue::Int(unsafe {
                // SAFETY: No values are borrowed.
                self.core
                    .api
                    .config
                    .get_param_int(self.handle, param.as_ptr())
            })),
            ConfigType::Float => Ok(ConfigValue::Float(unsafe {
                // SAFETY: No values are borrowed.
                self.core
                    .api
                    .config
                    .get_param_float(self.handle, param.as_ptr())
            })),
            ConfigType::Bool => Ok(ConfigValue::Bool(unsafe {
                // SAFETY: No values are borrowed.
                self.core
                    .api
                    .config
                    .get_param_bool(self.handle, param.as_ptr())
            })),
            ConfigType::String => Ok(ConfigValue::String(unsafe {
                // SAFETY: the pointer returned by ConfigGetParamString
                // is valid for an uncertain period of time. It is copied
                // to ensure that it won't be freed in that time.
                CStr::from_ptr(
                    self.core
                        .api
                        .config
                        .get_param_string(self.handle, param.as_ptr()),
                )
                .to_owned()
            })),
        }
    }

    /// Sets the value of a parameter. For convenience, you may pass in a value
    /// convertible to [`ConfigValue`].
    pub fn set<T: Into<ConfigValue>>(&mut self, param: &CStr, value: T) -> Result<(), M64PError> {
        let cfg_value: ConfigValue = value.into();

        unsafe {
            let param_type = cfg_value.cfg_type();
            let param_value = cfg_value.as_ptr();

            // SAFETY: the parameter value pointer is valid during this call,
            // it should also point to a valid value of cfg_type.
            core_fn(self.core.api.config.set_parameter(
                self.handle,
                param.as_ptr(),
                param_type,
                param_value,
            ))?;
        }

        Ok(())
    }

    /// Sets or unsets the help text of a parameter.
    pub fn set_help(&mut self, param: &CStr, help: Option<&CStr>) -> Result<(), M64PError> {
        core_fn(unsafe {
            // SAFETY: the two string pointers passed here are only used within
            // the function, they are not stored beyond that.
            self.core.api.config.set_parameter_help(
                self.handle,
                param.as_ptr(),
                help.map(|help| help.as_ptr()).unwrap_or(null()),
            )
        })
    }
}

/// Represents the value of a config parameter.
#[derive(Debug, Clone)]
#[repr(u32)]
pub enum ConfigValue {
    Int(c_int) = ConfigType::Int as u32,
    Float(c_float) = ConfigType::Float as u32,
    Bool(bool) = ConfigType::Bool as u32,
    String(CString) = ConfigType::String as u32,
}

impl ConfigValue {
    /// Returns the equivalent [`ConfigType`] for this value.
    pub fn cfg_type(&self) -> ConfigType {
        match self {
            ConfigValue::Int(_) => ConfigType::Int,
            ConfigValue::Float(_) => ConfigType::Float,
            ConfigValue::Bool(_) => ConfigType::Bool,
            ConfigValue::String(_) => ConfigType::String,
        }
    }

    /// (INTERNAL) Obtains a pointer to this value's data.
    pub(crate) unsafe fn as_ptr(&self) -> *const c_void {
        match self {
            ConfigValue::Int(value) => value as *const c_int as *const c_void,
            ConfigValue::Float(value) => value as *const c_float as *const c_void,
            ConfigValue::Bool(value) => value as *const bool as *const c_void,
            ConfigValue::String(value) => value.as_ptr() as *const c_void,
        }
    }
}

impl From<c_int> for ConfigValue {
    fn from(value: c_int) -> Self {
        Self::Int(value)
    }
}

impl From<c_float> for ConfigValue {
    fn from(value: c_float) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<CString> for ConfigValue {
    fn from(value: CString) -> Self {
        Self::String(value)
    }
}

impl TryInto<c_int> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<c_int, Self::Error> {
        match self {
            ConfigValue::Int(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Int, other.cfg_type())),
        }
    }
}

impl TryInto<c_float> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<c_float, Self::Error> {
        match self {
            ConfigValue::Float(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Float, other.cfg_type())),
        }
    }
}

impl TryInto<bool> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            ConfigValue::Bool(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Bool, other.cfg_type())),
        }
    }
}

impl TryInto<CString> for ConfigValue {
    type Error = WrongConfigType;

    fn try_into(self) -> Result<CString, Self::Error> {
        match self {
            ConfigValue::String(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::String, other.cfg_type())),
        }
    }
}
