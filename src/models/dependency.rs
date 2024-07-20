use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dependency {
    pub prev_task_id: i32,
    pub next_task_id: i32,
}
