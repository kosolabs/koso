use anyhow::{anyhow, Result};
use yrs::Origin;

pub struct YOrigin {
    pub who: String,
    pub id: String,
}

pub fn from_origin(origin: Option<&Origin>) -> Result<YOrigin> {
    origin
        .map(|o| match String::from_utf8(o.as_ref().to_vec()) {
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

pub fn as_origin(who: &str, id: &str) -> Origin {
    format!("{who}@@{id}").into()
}
