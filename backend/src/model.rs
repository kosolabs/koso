#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct NewTask {
    pub name: String,
}
