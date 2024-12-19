use std::{ffi::CString, fmt, marker::PhantomData, ops::Deref};

use log::Log;

use super::Core;

/// An implementation of [`log::Log`] using Mupen64Plus's logging methods.
pub struct CoreLogger<F, T>
where
    F: Fn() -> T + Send + Sync,
    T: Deref<Target = Core>,
{
    f: F,
    marker: PhantomData<fn() -> T>,
}

impl<F, T> CoreLogger<F, T>
where
    F: Fn() -> T + Send + Sync,
    T: Deref<Target = Core>,
{
    /// An implementation of [`log::Log`] using Mupen64Plus's logging system.
    pub fn new(f: F) -> Self {
        Self {
            f,
            marker: PhantomData,
        }
    }
}

impl<F, T> Log for CoreLogger<F, T>
where
    F: Fn() -> T + Send + Sync,
    T: Deref<Target = Core>,
{
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mesage_str = fmt::format(*record.args()).replace('\0', "[nul]");
        let message = CString::new(mesage_str).unwrap();

        let level = match record.level() {
            log::Level::Error => m64prs_sys::MsgLevel::Error,
            log::Level::Warn => m64prs_sys::MsgLevel::Warning,
            log::Level::Info => m64prs_sys::MsgLevel::Info,
            log::Level::Debug => m64prs_sys::MsgLevel::Status,
            log::Level::Trace => m64prs_sys::MsgLevel::Verbose,
        };

        let core_ref = (self.f)();
        core_ref.debug_message(level, &message);
    }

    fn flush(&self) {}
}
