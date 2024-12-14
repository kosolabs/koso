use super::User;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct AwarenessUpdate {
    client_id: i64,
    sequence: i64,
    selected: Vec<String>,
}

impl AwarenessUpdate {
    pub(super) fn into_state(self, user: User) -> AwarenessState {
        AwarenessState {
            client_id: self.client_id,
            sequence: self.sequence,
            selected: self.selected,
            user,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct AwarenessState {
    client_id: i64,
    sequence: i64,
    selected: Vec<String>,
    user: User,
}
