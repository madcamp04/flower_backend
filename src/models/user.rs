use sqlx::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub password_hash: String,
}
