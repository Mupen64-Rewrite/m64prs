
// CONFIGURATION API
// ===================

use std::{ffi::{c_char, c_void, CStr, CString}, ptr::{null, null_mut}};

use m64prs_sys::ConfigType;

use crate::{error::M64PError, types::ConfigValue};

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
