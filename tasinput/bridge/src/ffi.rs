use std::sync::Mutex;

use dlopen2::wrapper::WrapperApi;
use m64prs_sys::api::FullCoreApi;

pub(crate) struct PluginState {}

impl PluginState {
    pub(crate) fn init() {}
}

pub(crate) static INSTANCE: Mutex<Option<PluginState>> = Mutex::new(None);
