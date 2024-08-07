pub type ProjectId = String;

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Project {
    #[serde(default)]
    pub project_id: String,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct ProjectPermission {
    pub project_id: ProjectId,
    pub email: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct ProjectUser {
    pub project_id: ProjectId,
    pub email: String,
    pub name: String,
    pub picture: String,
}
