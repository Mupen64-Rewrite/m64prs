use glib::DateTime;

pub(super) struct PluginDatabase {}

pub(super) struct PluginInfo {
    filename: String,
    last_time: DateTime,
}
