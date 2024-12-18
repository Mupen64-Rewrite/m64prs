use std::sync::Mutex;


pub(crate) struct PluginState {}

impl PluginState {
    pub(crate) fn init() -> Result<Self, m64prs_sys::Error> {
        todo!()
    }
}

pub(crate) static INSTANCE: Mutex<Option<PluginState>> = Mutex::new(None);
