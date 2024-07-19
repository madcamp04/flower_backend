use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct User {
    pub user_id: i32,
    pub username: String,
}
