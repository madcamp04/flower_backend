use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub tag_id: i32,
    pub group_id: i32,
    pub tag_name: String,
    pub tag_color: String,
}
