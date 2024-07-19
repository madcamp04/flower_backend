use serde::Serialize;
use chrono::NaiveDateTime;

#[derive(sqlx::FromRow, Serialize)]
pub struct Task {
    pub task_id: i32,
    pub project_id: Option<i32>,
    pub worker_id: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}
