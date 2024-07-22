use serde::{Deserialize, Serialize};

// #[derive(Deserialize)]
// pub struct CheckUsernameRequest {
//     pub username: String,
// }

#[derive(Serialize)]
pub struct AdminDefaultResponse {
    pub success: bool,
    pub message: String,
}
