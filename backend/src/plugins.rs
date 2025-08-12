mod config;
pub mod github;
pub mod google_workspace;

#[derive(Default, Clone)]
pub(crate) struct PluginSettings {
    pub(crate) disable_polling: bool,
}
