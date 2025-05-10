use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use yrs::Origin;

use crate::api::google::User;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Actor {
    None,
    User(User),
    GitHub,
    Server,
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

    pub(crate) fn delegated(&self, prefix: &str) -> YOrigin {
        YOrigin {
            who: format!("{}-{}", prefix, self.who),
            id: format!("{}-{}", prefix, self.id),
            actor: Actor::Server,
        }
    }
}
