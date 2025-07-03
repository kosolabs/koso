use anyhow::{Context, Result, anyhow};
use config::{Environment, File, FileFormat};
use regex::Regex;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct Settings {
    pub(crate) env: String,
    pub(crate) host: String,
    pub(crate) database_url: String,
    pub(crate) secrets_dir: String,
    pub(crate) plugins: Plugins,
    pub(crate) stripe: Stripe,
    pub(crate) debug_path: Option<Regex>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsRaw {
    pub(crate) env: String,
    pub(crate) host: String,
    pub(crate) database_url: String,
    pub(crate) secrets_dir: String,
    pub(crate) plugins: Plugins,
    pub(crate) stripe: Stripe,
    pub(crate) debug_path: Option<String>,
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
    pub(crate) app_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stripe {
    pub(crate) price_id: String,
    pub(crate) enable_unauthenticated_webhook: bool,
}

pub fn settings() -> &'static Settings {
    static SETTINGS: OnceLock<Settings> = OnceLock::new();
    SETTINGS.get_or_init(|| {
        load_settings_from_env()
            .context("load_settings_from_env failed")
            .unwrap()
    })
}

fn load_settings_from_env() -> Result<Settings> {
    load_settings(&std::env::var("KOSO_ENV").context("KOSO_ENV is unset")?)
}

fn load_settings(env: &str) -> Result<Settings> {
    let raw: SettingsRaw = config::Config::builder()
        .add_source(match env {
            "dev" => File::from_str(include_str!("settings/dev.json"), FileFormat::Json),
            "prod" => File::from_str(include_str!("settings/prod.json"), FileFormat::Json),
            env => {
                return Err(anyhow!(
                    "No settings file for '{env}' found. Expected 'dev' or 'prod'."
                ));
            }
        })
        .add_source(File::new(".local_settings", FileFormat::Json).required(false))
        .add_source(Environment::with_prefix("KOSO_SETTING"))
        .build()
        .context("Failed to load settings")?
        .try_deserialize()
        .context("Failed to deserialize settings")?;

    let debug_path = raw
        .debug_path
        .map(|pattern| Regex::new(&pattern))
        .transpose()?;

    Ok(Settings {
        env: raw.env,
        host: raw.host,
        database_url: raw.database_url,
        secrets_dir: raw.secrets_dir,
        plugins: raw.plugins,
        stripe: raw.stripe,
        debug_path,
    })
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
        let s = load_settings("dev").unwrap();
        assert_eq!(s.env, "dev");
    }

    #[test_log::test]
    fn prod_settings_test() {
        let s = load_settings("prod").unwrap();
        assert_eq!(s.env, "prod", "Got {}", s.env);
    }

    #[test_log::test]
    fn load_env_settings_test() {
        let s = load_settings_from_env().unwrap();
        assert_eq!(s.env, "dev");
    }
}
