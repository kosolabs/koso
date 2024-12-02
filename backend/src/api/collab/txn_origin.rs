use anyhow::{anyhow, Result};
use yrs::Origin;

pub(crate) struct YOrigin {
    pub(crate) who: String,
    pub(crate) id: String,
}

pub(crate) fn from_origin(origin: Option<&Origin>) -> Result<YOrigin> {
    origin
        .map(|o| match core::str::from_utf8(o.as_ref()) {
            Ok(v) => {
                let mut parts = v.split("@@");
                let (Some(who), Some(id)) = (parts.next(), parts.next()) else {
                    return Err(anyhow!("Could not split origin into parts: {v}"));
                };
                Ok(YOrigin {
                    who: who.to_string(),
                    id: id.to_string(),
                })
            }
            Err(e) => Err(anyhow!("Failed to parse origin bytes to string: {o}: {e}")),
        })
        .unwrap_or_else(|| Err(anyhow!("Missing origin")))
}

impl YOrigin {
    pub(crate) fn as_origin(&self) -> Origin {
        format!("{}@@{}", self.who, self.id).into()
    }
}
