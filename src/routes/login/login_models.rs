use serde::{Deserialize, Serialize};

// Username check request and response
#[derive(Deserialize)]
pub struct CheckUsernameRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    pub is_unique: bool,
}


// Email check request and response
#[derive(Deserialize)]
pub struct CheckEmailRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct CheckEmailResponse {
    pub is_unique: bool,
}


// Registration request and response
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
}


// Login request and response
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
}


// Auto-login request and response
#[derive(Deserialize)]
pub struct AutoLoginRequest {
}

#[derive(Serialize)]
pub struct AutoLoginResponse {
    pub success: bool,
    pub message: String,
}


// Logout request and response
#[derive(Deserialize)]
pub struct LogoutRequest {
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}
