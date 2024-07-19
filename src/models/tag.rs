use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Tag {
    pub tag_id: i32,
    pub pm_id: Option<i32>,
    pub tag_name: String,
}
