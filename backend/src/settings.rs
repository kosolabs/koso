use config::{Environment, File, FileFormat};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub(crate) env: String,
    pub(crate) database_url: String,
    pub(crate) port: u16,
    pub(crate) secrets_dir: String,
    pub(crate) plugins: Plugins,
    pub(crate) metrics: Metrics,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plugins {
    pub(crate) github: Github,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Github {
    pub(crate) client_id: String,
    pub(crate) app_id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Metrics {
    pub(crate) port: u16,
}

pub fn settings() -> &'static Settings {
    static SETTINGS: OnceLock<Settings> = OnceLock::new();
    SETTINGS.get_or_init(load_settings_from_env)
}

fn load_settings_from_env() -> Settings {
    load_settings(&std::env::var("RUN_KOSO_ENV").expect("RUN_KOSO_ENV is unset"))
}

fn load_settings(env: &str) -> Settings {
    let env_source = match env {
        "dev" => File::from_str(include_str!("settings/dev.toml"), FileFormat::Toml),
        "prod" => File::from_str(include_str!("settings/prod.toml"), FileFormat::Toml),
        env => panic!("No settings file for '{env}' found. Expected 'dev' or 'prod'."),
    };

    let config = config::Config::builder()
        .add_source(env_source)
        .add_source(File::new(".local_settings", FileFormat::Toml).required(false))
        .add_source(Environment::with_prefix("KOSO"))
        .build()
        .expect("Failed to load settings");
    if std::env::var("RUN_KOSO_LOG_SETTINGS").unwrap_or_default() == "true" {
        println!("Using koso settings config: {config:?}");
    }

    let settings = config
        .try_deserialize()
        .expect("Failed to deserialize settings");
    if std::env::var("RUN_KOSO_LOG_SETTINGS").unwrap_or_default() == "true" {
        println!("Using koso settings: {settings:?}");
    }

    settings
}

impl Settings {
    pub fn is_dev(&self) -> bool {
        settings().env == "dev"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn dev_settings_test() {
        let s = load_settings("dev");
        assert_eq!(s.env, "dev");
    }

    #[test_log::test]
    fn prod_settings_test() {
        let s = load_settings("prod");
        assert_eq!(s.env, "prod", "Got {}", s.env);
    }

    #[test_log::test]
    fn load_env_settings_test() {
        let s = load_settings_from_env();
        assert_eq!(s.env, "dev");
    }
}
