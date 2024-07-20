use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub project_id: i32,
    pub group_id: i32,
    pub project_name: String,
    pub project_description: String,
}
