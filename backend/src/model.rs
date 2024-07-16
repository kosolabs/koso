#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub children: Vec<String>,
}
