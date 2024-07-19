use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Worker {
    pub worker_id: i32,
    pub worker_name: String,
    pub worker_user_id: Option<i32>,
}
