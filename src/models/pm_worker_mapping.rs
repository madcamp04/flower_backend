use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct PMWorkerMapping {
    pub pm_id: i32,
    pub worker_id: i32,
}
