use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TagProjectMapping {
    pub tag_id: i32,
    pub project_id: i32,
}
