use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct ProjectManager {
    pub pm_id: i32,
    pub pm_name: String,
    pub pm_user_id: Option<i32>,
}
