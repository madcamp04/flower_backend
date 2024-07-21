use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub group_id: i32,
    pub group_name: String,
    pub owner_user_id: i32,
}
