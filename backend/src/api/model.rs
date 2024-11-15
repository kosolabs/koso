use std::{collections::HashMap, fmt};

pub(crate) type ProjectId = String;

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub(crate) struct Project {
    #[serde(rename(serialize = "projectId", deserialize = "projectId"))]
    pub(crate) project_id: String,
    pub(crate) name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateProject {
    pub(crate) name: String,
    #[serde(rename(serialize = "projectExport", deserialize = "projectExport"))]
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
pub(crate) struct ProjectPermission {
    #[serde(rename(serialize = "projectId", deserialize = "projectId"))]
    pub(crate) project_id: ProjectId,
    pub(crate) email: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateProjectUsers {
    #[serde(rename(serialize = "projectId", deserialize = "projectId"))]
    pub project_id: ProjectId,
    #[serde(rename(serialize = "addEmails", deserialize = "addEmails"))]
    pub add_emails: Vec<String>,
    #[serde(rename(serialize = "removeEmails", deserialize = "removeEmails"))]
    pub remove_emails: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateProjectUsersResponse {}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub(crate) struct ProjectUser {
    #[serde(rename(serialize = "projectId", deserialize = "projectId"))]
    pub(crate) project_id: ProjectId,
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub(crate) struct User {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub(crate) struct ProjectExport {
    #[serde(rename(serialize = "projectId", deserialize = "projectId"))]
    pub(crate) project_id: ProjectId,
    pub(crate) graph: Graph,
}

pub(crate) type Graph = HashMap<String, Task>;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub(crate) struct Task {
    pub(crate) id: String,
    pub(crate) num: String,
    pub(crate) name: String,
    pub(crate) children: Vec<String>,
    pub(crate) assignee: Option<String>,
    pub(crate) reporter: Option<String>,
    pub(crate) status: Option<String>,
    #[serde(rename(serialize = "statusTime", deserialize = "statusTime"))]
    pub(crate) status_time: Option<i64>,
}
