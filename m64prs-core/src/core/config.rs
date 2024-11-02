// CONFIGURATION API
// ===================

use std::{
    ffi::{c_char, c_float, c_int, c_void, CStr, CString},
    ptr::{null, null_mut},
};

use m64prs_sys::ConfigType;

use crate::error::{M64PError, WrongConfigType};

use super::{core_fn, Core};

/// Functions for the configuration system.
impl Core {
    /// Runs the provided callback once per available config section.
    pub fn cfg_for_each_section<F: FnMut(&CStr)>(&self, mut callback: F) -> Result<(), M64PError> {
        unsafe extern "C" fn run_callback<F: FnMut(&CStr)>(
            context: *mut c_void,
            name: *const c_char,
        ) {
            let function: &mut F = &mut *(context as *mut F);
            function(CStr::from_ptr(name));
        }

        core_fn(unsafe {
            self.api
                .config
                .list_sections(&mut callback as *mut F as *mut c_void, run_callback::<F>)
        })?;

        Ok(())
    }
    /// Opens the config section with the given name.
    pub fn cfg_open(&self, name: &CStr) -> Result<ConfigSection, M64PError> {
        let mut handle: m64prs_sys::Handle = null_mut();
        core_fn(unsafe {
            self.api
                .config
                .open_section(name.as_ptr(), &mut handle as *mut m64prs_sys::Handle)
        })?;

        Ok(ConfigSection {
            core: self,
            name: name.to_owned(),
            handle: handle,
        })
    }
}
/// Represents a handle to a configuration section.
///
/// Each configuration section contains a list of parameters with values of variable type.
pub struct ConfigSection<'a> {
    core: &'a Core,
    name: CString,
    handle: m64prs_sys::Handle,
}

impl<'a> ConfigSection<'a> {
    /// Returns the section's name.
    pub fn name(&self) -> &CStr {
        &self.name
    }

    /// Saves the current section to disk.
    pub fn save(&self) -> Result<(), M64PError> {
        core_fn(unsafe { self.core.api.config.save_section(self.name.as_ptr()) })
    }

    /// Reverts any unsaved changes in this section.
    pub fn revert(&self) -> Result<(), M64PError> {
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
            let help_ptr = self
                .core
                .api
                .config
                .get_parameter_help(self.handle, param.as_ptr());

            if help_ptr == null() {
                return Err(M64PError::InputNotFound);
            } else {
                return Ok(CStr::from_ptr(help_ptr).to_owned());
            }
        }
    }

    /// Gets the value of a parameter.
    pub fn get(&self, param: &CStr) -> Result<ConfigValue, M64PError> {
        let param_type = self.get_type(param)?;

        match param_type {
            ConfigType::Int => Ok(ConfigValue::Int(unsafe {
                self.core
                    .api
                    .config
                    .get_param_int(self.handle, param.as_ptr())
            })),
            ConfigType::Float => Ok(ConfigValue::Float(unsafe {
                self.core
                    .api
                    .config
                    .get_param_float(self.handle, param.as_ptr())
            })),
            ConfigType::Bool => Ok(ConfigValue::Bool(unsafe {
                self.core
                    .api
                    .config
                    .get_param_bool(self.handle, param.as_ptr())
            })),
            ConfigType::String => Ok(ConfigValue::String(unsafe {
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
    pub fn set<T: Into<ConfigValue>>(&self, param: &CStr, value: T) -> Result<(), M64PError> {
        let cfg_value: ConfigValue = value.into();

        unsafe {
            let param_type = cfg_value.cfg_type();
            let param_value = cfg_value.as_ptr();

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
    pub fn set_help(&self, param: &CStr, help: Option<&CStr>) -> Result<(), M64PError> {
        core_fn(unsafe {
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
pub enum ConfigValue {
    Int(c_int),
    Float(c_float),
    Bool(bool),
    String(CString),
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
