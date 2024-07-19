use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct TagProjectMapping {
    pub tag_id: i32,
    pub project_id: i32,
}
