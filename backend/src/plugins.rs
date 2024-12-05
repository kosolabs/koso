mod config;
pub mod github;

#[derive(Default, Clone)]
pub(crate) struct PluginSettings {
    pub(crate) disable_polling: bool,
}
