use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub session_id: String,
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
    pub is_persistent: bool,
}
