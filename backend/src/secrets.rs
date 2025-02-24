use anyhow::{Context as _, Result, anyhow};
use core::fmt;
use std::{fmt::Debug, fs, path::Path};

use crate::settings::settings;

#[derive(Clone)]
pub(crate) struct Secret<T> {
    pub(crate) data: T,
}

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

/// Read the secret from $secrets_dir/$sub_path.
pub(crate) fn read_secret<T: std::convert::From<String>>(sub_path: &str) -> Result<Secret<T>> {
    let path = secret_path(sub_path)?;
    let secret: String = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read secret from {path}"))?
        .trim()
        .to_owned();
    Ok(Secret {
        data: secret.into(),
    })
}

pub(crate) fn secret_path(sub_path: &str) -> Result<String> {
    let dir = &settings().secrets_dir;
    Path::new(dir)
        .join(sub_path)
        .into_os_string()
        .into_string()
        .map_err(|p| anyhow!("Invalid secret path in {dir}: {p:?}"))
}
