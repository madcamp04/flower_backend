use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GroupUserMapping {
    pub group_id: i32,
    pub user_id: i32,
    pub writeable: bool,
}
