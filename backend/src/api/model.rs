use sqlx::types::chrono::{self, Utc};
use std::{collections::HashMap, fmt};

pub(crate) type ProjectId = String;

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Project {
    pub(crate) project_id: String,
    pub(crate) name: String,
    pub(crate) deleted_on: Option<chrono::DateTime<Utc>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateProject {
    pub(crate) name: String,
    pub(crate) project_export: Option<ProjectExport>,
}

impl fmt::Debug for CreateProject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateProject")
            .field("name", &self.name)
            .finish()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectPermission {
    pub(crate) project_id: ProjectId,
    pub(crate) email: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectUsers {
    pub project_id: ProjectId,
    pub add_emails: Vec<String>,
    pub remove_emails: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateProjectUsersResponse {}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectUser {
    pub(crate) project_id: ProjectId,
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
    pub(crate) premium: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
    pub(crate) premium: bool,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectExport {
    pub(crate) project_id: ProjectId,
    pub(crate) graph: Graph,
}

pub(crate) type Graph = HashMap<String, Task>;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Task {
    pub(crate) id: String,
    pub(crate) num: String,
    pub(crate) name: String,
    pub(crate) desc: Option<String>,
    pub(crate) children: Vec<String>,
    pub(crate) assignee: Option<String>,
    pub(crate) reporter: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) status_time: Option<i64>,
    pub(crate) url: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) estimate: Option<i64>,
    pub(crate) deadline: Option<i64>,
    pub(crate) archived: Option<bool>,
}

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::api::model::Task;

    pub(crate) fn new_with_fields_populated() -> Task {
        // Populate all fields with non-null, non-empty values for testing.
        Task {
            id: "id1".to_string(),
            num: "1".to_string(),
            name: "Task 1".to_string(),
            desc: Some("Task 1 description".to_string()),
            children: vec!["2".to_string()],
            assignee: Some("a@gmail.com".to_string()),
            reporter: Some("r@gmail.com".to_string()),
            status: Some("Done".to_string()),
            status_time: Some(1),
            url: Some("https://example.com/1".to_string()),
            kind: Some("Kind1".to_string()),
            estimate: Some(0),
            deadline: Some(152),
            archived: Some(false),
        }
    }
}
