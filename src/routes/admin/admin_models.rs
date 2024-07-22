use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminDefaultRequest {
}

#[derive(Serialize)]
pub struct AdminDefaultResponse {
    pub success: bool,
    pub message: String,
}
