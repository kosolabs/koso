use anyhow::{anyhow, Result};
use core::fmt;
use std::{fmt::Debug, fs, path::Path};

#[derive(Clone)]
pub(crate) struct Secret<T> {
    pub(crate) data: T,
}

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

const DEFAULT_SECRETS_DIR: &str = "../.secrets";

/// Read the secret from $secrets_dir/$sub_path.
/// The default is `../.secrets/$sub_path`, unless `SECRETS_DIR` is set.
pub(crate) fn read_secret<T: std::convert::From<String>>(sub_path: &str) -> Result<Secret<T>> {
    let dir = std::env::var("SECRETS_DIR").unwrap_or_else(|_| DEFAULT_SECRETS_DIR.to_string());
    let path = Path::new(&dir)
        .join(sub_path)
        .into_os_string()
        .into_string()
        .map_err(|e| anyhow!("Invalid secret path in {dir}: {e:?}"))?;
    tracing::info!("Using {sub_path} secret at {path}");
    let secret: String = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read secret from {path}: {e}"))?
        .trim()
        .to_owned();
    Ok(Secret {
        data: secret.into(),
    })
}
