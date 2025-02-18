use anyhow::{Context as _, Result, anyhow};
use core::fmt;
use std::{
    fmt::Debug,
    fs,
    io::ErrorKind,
    path::{self, Path},
};

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

/// Read the secret from $secrets_dir/$sub_path, if present.
pub(crate) fn read_optional_secret<T: std::convert::From<String>>(
    sub_path: &str,
) -> Result<Option<Secret<T>>> {
    read(&secret_path(sub_path)?)
}

/// Read the secret from $secrets_dir/$sub_path.
pub(crate) fn read_secret<T: std::convert::From<String>>(sub_path: &str) -> Result<Secret<T>> {
    let path = secret_path(sub_path)?;
    read::<T>(&path)?.with_context(|| format!("Secret at {path} not found."))
}

fn read<T: std::convert::From<String>>(path: &str) -> Result<Option<Secret<T>>> {
    let secret: String = match fs::read_to_string(&path) {
        Ok(s) => s.trim().to_owned(),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(anyhow!("Failed to read secret from {path}: {e}"));
        }
    };
    Ok(Some(Secret {
        data: secret.into(),
    }))
}

pub(crate) fn secret_path(sub_path: &str) -> Result<String> {
    let dir = &settings().secrets_dir;
    Path::new(dir)
        .join(sub_path)
        .into_os_string()
        .into_string()
        .map_err(|p| anyhow!("Invalid secret path in {dir}: {p:?}"))
}
