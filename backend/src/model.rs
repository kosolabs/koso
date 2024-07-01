#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub children: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct NewTask {
    pub name: String,
    pub children: Vec<String>,
}
