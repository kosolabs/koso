#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub children: Vec<String>,
    pub assignee: Option<String>,
    pub reporter: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Project {
    pub project_id: String,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct ProjectPermission {
    pub project_id: String,
    pub email: String,
}
