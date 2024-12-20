use m64prs_plugin_core::Core;

pub(crate) struct PluginState {
    core: Core,
}

impl PluginState {
    pub(crate) fn init(core: Core) -> Result<Self, m64prs_sys::Error> {
        Ok(Self { core })
    }
}