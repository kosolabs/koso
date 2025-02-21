use crate::api::model::User;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use yrs::Origin;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Actor {
    None,
    User(User),
    GitHub,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct YOrigin {
    pub(crate) who: String,
    pub(crate) id: String,
    pub(crate) actor: Actor,
}

pub(crate) fn from_origin(origin: Option<&Origin>) -> Result<YOrigin> {
    let Some(origin) = origin else {
        return Err(anyhow!("Missing origin"));
    };
    Ok(serde_json::from_slice(origin.as_ref())?)
}

impl YOrigin {
    pub(crate) fn as_origin(&self) -> Result<Origin> {
        Ok(serde_json::to_string(self)?.into())
    }
}
