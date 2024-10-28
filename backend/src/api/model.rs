pub(crate) type ProjectId = String;

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub(crate) struct Project {
    pub(crate) project_id: String,
    pub(crate) name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub(crate) struct CreateProject {
    pub(crate) name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub(crate) struct ProjectPermission {
    pub(crate) project_id: ProjectId,
    pub(crate) email: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateProjectUsers {
    pub project_id: ProjectId,
    pub add_emails: Vec<String>,
    pub remove_emails: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateProjectUsersResponse {}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub(crate) struct ProjectUser {
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

#[derive(serde::Serialize, Debug)]
pub(crate) struct ProjectExport {
    pub(crate) project_id: ProjectId,
    pub(crate) data: yrs::Any,
}
