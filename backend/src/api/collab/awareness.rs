use serde::{Deserialize, Serialize};

use crate::api::google::User;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AwarenessUpdate {
    client_id: i64,
    sequence: i64,
    selected: Vec<String>,
}

impl AwarenessUpdate {
    pub(crate) fn into_state(self, user: &User) -> AwarenessState {
        AwarenessState {
            client_id: self.client_id,
            sequence: self.sequence,
            selected: self.selected,
            user: AwarenessUser {
                email: user.email.clone(),
                name: user.name.clone(),
                picture: user.picture.clone(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AwarenessState {
    client_id: i64,
    sequence: i64,
    selected: Vec<String>,
    user: AwarenessUser,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AwarenessUser {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
}
