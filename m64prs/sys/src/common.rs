//! Common safe types for Mupen64Plus.

use std::{
    error::Error,
    ffi::{c_float, c_int, c_uint, c_void, CStr, CString},
    fmt::Display,
};

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

use crate::ConfigType;

mod sealed {
    pub trait Sealed {}
}

/// Safer representation of a Mupen64Plus error which always represents an error state.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error, IntoPrimitive, TryFromPrimitive)]
pub enum M64PError {
    /// A function was called before it's associated module was initialized.
    #[error("A function was called before it's associated module was initialized")]
    NotInit = crate::Error::NotInit as u32,
    /// An initialization function was called twice.
    #[error("Initialization function called twice")]
    AlreadyInit = crate::Error::AlreadyInit as u32,
    /// API versions between components are incompatible
    #[error("API versions between components are incompatible")]
    Incompatible = crate::Error::Incompatible as u32,
    /// Invalid function parameter. Returned for trivial assertions, such as a pointer being non-null.
    #[error("Invalid function parameters, such as a NULL pointer")]
    InputAssert = crate::Error::InputAssert as u32,
    /// Invalid function parameter. Returned for non-trivial assertions.
    #[error("An input function parameter is logically invalid")]
    InputInvalid = crate::Error::InputInvalid as u32,
    /// A function parameter (such as a configuration key) was not found.
    #[error("The input parameter(s) specified a particular item which was not found")]
    InputNotFound = crate::Error::InputNotFound as u32,
    /// Memory allocation failed.
    #[error("Memory allocation failed")]
    NoMemory = crate::Error::NoMemory as u32,
    /// An error occurred when working with a file.
    #[error("Error opening, creating, reading, or writing to a file")]
    Files = crate::Error::Files as u32,
    /// An internal error.
    #[error("Logical inconsistency in program code. Probably a bug.")]
    Internal = crate::Error::Internal as u32,
    /// This function's module is not in the correct state to do this action.
    #[error("An operation was requested which is not allowed in the current state")]
    InvalidState = crate::Error::InvalidState as u32,
    /// A plugin caused a fatal error.
    #[error("A plugin function returned a fatal error")]
    PluginFail = crate::Error::PluginFail as u32,
    /// A system library caused an error.
    #[error("A system function call, such as an SDL or file operation, failed")]
    SystemFail = crate::Error::SystemFail as u32,
    /// The specified function is either unsupported or not compiled in.
    #[error(
        "Function call or argument is not supported (e.g. no debugger, invalid encoder format)"
    )]
    Unsupported = crate::Error::Unsupported as u32,
    /// A parameter representing a type is invalid for this operation.
    #[error("A given input type parameter cannot be used for desired operation")]
    WrongType = crate::Error::WrongType as u32,
}

impl TryFrom<crate::Error> for M64PError {
    type Error = TryFromPrimitiveError<M64PError>;

    fn try_from(value: crate::Error) -> std::result::Result<Self, Self::Error> {
        let prim = value as u32;
        prim.try_into()
    }
}

impl From<M64PError> for crate::Error {
    fn from(value: M64PError) -> Self {
        // SAFETY: every value of M64PError always corresponds to a value of crate::Error.
        unsafe { std::mem::transmute(value) }
    }
}

/// Error returned when a the wrong config type is specified.
#[derive(Debug)]
pub struct WrongConfigType {
    expected: ConfigType,
    actual: ConfigType,
}

impl WrongConfigType {
    pub fn new(expected: ConfigType, actual: ConfigType) -> Self {
        Self { expected, actual }
    }
}

impl Display for WrongConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "parameter type is {} (expected {})",
            self.actual, self.expected
        ))
    }
}

impl Error for WrongConfigType {}

/// Represents the value of a config parameter.
#[derive(Debug, Clone)]
#[repr(u32)]
pub enum ConfigValue {
    /// An integer value that fits in the C `int` type.
    Int(c_int) = ConfigType::Int as u32,
    /// A floating-point value that fits in the C `float` type.
    Float(c_float) = ConfigType::Float as u32,
    /// A boolean value, stored as a C `uint`; this is zero when false and nonzero when true.
    ///
    /// # Notes
    /// This value is stored as an integer since Mupen passes boolean
    /// parameters using `int` instead of using `bool` like modern C/C++ would.
    Bool(bool) = ConfigType::Bool as u32,
    /// A null-terminated string, typically exposed to C as `const char*`.
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
}

pub trait ConfigSettable {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error;
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error;
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

impl TryFrom<ConfigValue> for c_int {
    type Error = WrongConfigType;

    fn try_from(value: ConfigValue) -> Result<Self, Self::Error> {
        match value {
            ConfigValue::Int(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Int, other.cfg_type())),
        }
    }
}
impl TryFrom<ConfigValue> for c_float {
    type Error = WrongConfigType;

    fn try_from(value: ConfigValue) -> Result<Self, Self::Error> {
        match value {
            ConfigValue::Float(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Float, other.cfg_type())),
        }
    }
}
impl TryFrom<ConfigValue> for bool {
    type Error = WrongConfigType;

    fn try_from(value: ConfigValue) -> Result<Self, Self::Error> {
        match value {
            ConfigValue::Bool(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::Bool, other.cfg_type())),
        }
    }
}
impl TryFrom<ConfigValue> for CString {
    type Error = WrongConfigType;

    fn try_from(value: ConfigValue) -> Result<Self, Self::Error> {
        match value {
            ConfigValue::String(value) => Ok(value),
            other => Err(WrongConfigType::new(ConfigType::String, other.cfg_type())),
        }
    }
}

impl<T> ConfigSettable for &T
where
    T: ConfigSettable,
{
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        T::set(*self, config, handle, key)
    }

    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        T::set_default(*self, config, handle, key, help)
    }
}

impl ConfigSettable for c_int {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        (config.set_parameter)(
            handle,
            key.as_ptr(),
            ConfigType::Int,
            self as *const c_int as *const c_void,
        )
    }
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        (config.set_default_int)(handle, key.as_ptr(), *self, help.as_ptr())
    }
}
impl ConfigSettable for c_float {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        (config.set_parameter)(
            handle,
            key.as_ptr(),
            ConfigType::Float,
            self as *const c_float as *const c_void,
        )
    }
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        (config.set_default_float)(handle, key.as_ptr(), *self, help.as_ptr())
    }
}
impl ConfigSettable for bool {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        let value_i32 = *self as c_int;
        (config.set_parameter)(
            handle,
            key.as_ptr(),
            ConfigType::Bool,
            &value_i32 as *const c_int as *const c_void,
        )
    }
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        (config.set_default_bool)(handle, key.as_ptr(), *self as i32, help.as_ptr())
    }
}
impl ConfigSettable for CString {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        (config.set_parameter)(
            handle,
            key.as_ptr(),
            ConfigType::String,
            self.as_ptr() as *const c_void,
        )
    }
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        (config.set_default_string)(handle, key.as_ptr(), self.as_ptr(), help.as_ptr())
    }
}
impl ConfigSettable for CStr {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        (config.set_parameter)(
            handle,
            key.as_ptr(),
            ConfigType::String,
            self.as_ptr() as *const c_void,
        )
    }
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        (config.set_default_string)(handle, key.as_ptr(), self.as_ptr(), help.as_ptr())
    }
}
impl ConfigSettable for ConfigValue {
    unsafe fn set(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
    ) -> crate::Error {
        match self {
            ConfigValue::Int(value) => value.set(config, handle, key),
            ConfigValue::Float(value) => value.set(config, handle, key),
            ConfigValue::Bool(value) => value.set(config, handle, key),
            ConfigValue::String(value) => value.set(config, handle, key),
        }
    }
    unsafe fn set_default(
        &self,
        config: &crate::api::CoreConfigApi,
        handle: crate::Handle,
        key: &CStr,
        help: &CStr,
    ) -> crate::Error {
        match self {
            ConfigValue::Int(value) => value.set_default(config, handle, key, help),
            ConfigValue::Float(value) => value.set_default(config, handle, key, help),
            ConfigValue::Bool(value) => value.set_default(config, handle, key, help),
            ConfigValue::String(value) => value.set_default(config, handle, key, help),
        }
    }
}
