use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Dependency {
    pub prev_task_id: i32,
    pub task_id: i32,
}
