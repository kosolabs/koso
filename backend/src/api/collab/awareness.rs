use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Awareness {
    client_id: i64,
    sequence: i64,
    selected: Option<String>,
}
