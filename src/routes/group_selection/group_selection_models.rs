use serde::{Deserialize, Serialize};

// Username check request and response
#[derive(Deserialize)]
pub struct GetGroupListRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct GetGroupListResponse {
    pub is_unique: bool,
}