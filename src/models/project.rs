use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Project {
    pub project_id: i32,
    pub pm_id: Option<i32>,
    pub project_name: String,
}
