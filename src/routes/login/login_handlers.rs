use actix_web::{web, HttpResponse, HttpRequest, Responder};
use sqlx::MySqlPool;
use log::{error, info};
use uuid::Uuid;
use chrono::{Utc, Duration};
use time::OffsetDateTime;
use bcrypt::{hash, DEFAULT_COST, verify};
use super::login_models::{
    CheckUsernameRequest, CheckUsernameResponse,
    CheckEmailRequest, CheckEmailResponse,
    RegisterRequest, RegisterResponse,
    LoginRequest, LoginResponse,
    AutoLoginRequest, AutoLoginResponse,
    LogoutRequest, LogoutResponse,
};

pub async fn login_get() -> impl Responder {
    info!("Received request on /login_get endpoint");
    HttpResponse::Ok().body("Hello this is Flow'er's Login endpoint.")
}

// Check if username is unique
pub async fn check_username(
    pool: web::Data<MySqlPool>,
    req: web::Json<CheckUsernameRequest>,
) -> impl Responder {
    let username = &req.username;
    info!("Received request to check username: {}", username);
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM Users_ WHERE user_name = ?",
        username
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            let is_unique = record.count == 0;
            info!("Username {} is unique: {}", username, is_unique);
            HttpResponse::Ok().json(CheckUsernameResponse { is_unique })
        }
        Err(e) => {
            error!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Check if email is unique
pub async fn check_email(
    pool: web::Data<MySqlPool>,
    req: web::Json<CheckEmailRequest>,
) -> impl Responder {
    let email = &req.email;
    info!("Received request to check email: {}", email);
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM Users_ WHERE user_email = ?",
        email
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            let is_unique = record.count == 0;
            info!("Email {} is unique: {}", email, is_unique);
            HttpResponse::Ok().json(CheckEmailResponse { is_unique })
        }
        Err(e) => {
            error!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// register user to DB
pub async fn register(
    pool: web::Data<MySqlPool>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let username = &req.username;
    let email = &req.email;
    let password = &req.password;
    info!("Received request to register user: {}", username);
    
    // Encrypt password with bcrypt
    let hashed_password = match hash(password, DEFAULT_COST) {
        Ok(hp) => hp,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError().json(RegisterResponse {
                success: false,
                message: "Failed to hash password".into(),
            });
        }
    };

    // Insert username, email, hashed_password into Users_ table
    let result = sqlx::query!(
        "INSERT INTO Users_ (user_name, user_email, password_hash) VALUES (?, ?, ?)",
        username, email, hashed_password
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            info!("User {} registered successfully", username);
            HttpResponse::Ok().json(RegisterResponse {
                success: true,
                message: "User registered successfully".into(),
            })
        }
        Err(e) => {
            error!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().json(RegisterResponse {
                success: false,
                message: "Failed to register user".into(),
            })
        }
    }
}

// login logic
pub async fn login(
    pool: web::Data<MySqlPool>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    let username = &req.username;
    let password = &req.password;
    info!("Received login request for user: {}", username);

    // 2. Get the user data from the database with username
    let result = sqlx::query!(
        "SELECT user_id, password_hash FROM Users_ WHERE user_name = ?",
        username
    )
    .fetch_one(pool.get_ref())
    .await;

    let user = match result {
        Ok(user) => user,
        Err(_) => {
            info!("Invalid username: {}", username);
            return HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                message: "Invalid username".into(),
            });
        }
    };

    // 3. Validate hashed password in DB and given password
    let valid = match verify(password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            error!("Error when checking password for user: {}", username);
            return HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                message: "Error when checking password".into(),
            });
        }
    };

    if !valid {
        info!("Invalid password for user: {}", username);
        return HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            message: "Invalid password".into(),
        });
    }

    // 4. Generate a new session ID
    let new_session_id = Uuid::new_v4().to_string();
    let expires_at = if req.remember_me {
        Utc::now() + Duration::days(10)
    } else {
        Utc::now() + Duration::minutes(30)
    };

    // 5. Check if user already has a session
    let session_check = sqlx::query!(
        "SELECT session_id, expires_at FROM Sessions_ WHERE user_id = ?",
        user.user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match session_check {
        Ok(Some(session)) => {
            // Session exists, check if it has expired
            if OffsetDateTime::now_utc() < session.expires_at {
                info!("User {} already has an active session", username);
                return HttpResponse::BadRequest().json(LoginResponse {
                    success: false,
                    message: "User already has an active session".into(),
                });
            } else {
                // Update the expired session with a new session ID and expiration
                let update_result = sqlx::query!(
                    "UPDATE Sessions_ SET session_id = ?, expires_at = ?, is_persistent = ? WHERE user_id = ?",
                    new_session_id,
                    expires_at,
                    req.remember_me,
                    user.user_id
                )
                .execute(pool.get_ref())
                .await;

                if let Err(e) = update_result {
                    error!("Failed to update session for user {}: {}", username, e);
                    return HttpResponse::InternalServerError().json(LoginResponse {
                        success: false,
                        message: "Failed to update session".into(),
                    });
                }
            }
        }
        Ok(None) => {
            // No active session found, insert a new one
            let insert_result = sqlx::query!(
                "INSERT INTO Sessions_ (session_id, user_id, expires_at, is_persistent) VALUES (?, ?, ?, ?)",
                new_session_id,
                user.user_id,
                expires_at,
                req.remember_me
            )
            .execute(pool.get_ref())
            .await;

            if let Err(e) = insert_result {
                error!("Failed to insert session for user {}: {}", username, e);
                return HttpResponse::InternalServerError().json(LoginResponse {
                    success: false,
                    message: "Failed to create session".into(),
                });
            }
        }
        Err(e) => {
            error!("Failed to query session for user {}: {}", username, e);
            return HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                message: "Failed to check session".into(),
            });
        }
    };

    // 6. Return session ID inside a cookie to the client
    info!("User {} logged in successfully", username);
    HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("session_id", new_session_id.clone())
                .http_only(true)
                .finish(),
        )
        .json(LoginResponse {
            success: true,
            message: "Login successful".into(),
        })
}

// auto_login logic
pub async fn auto_login(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    _: web::Json<AutoLoginRequest>,
) -> impl Responder {
    // 1. Receive the session ID from the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for auto login");
            return HttpResponse::BadRequest().json(AutoLoginResponse {
                success: false,
                message: "Session ID not found in cookies".into(),
                username: "".into(),
            });
        }
    };

    info!("Received auto login request with session ID: {}", session_id);

    // 2. Check whether session ID is valid (by fetching the session from Session table using session ID)
    let session_result = sqlx::query!(
        "SELECT user_id, expires_at FROM Sessions_ WHERE session_id = ?",
        session_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match session_result {
        Ok(Some(session)) => {
            // Check if the session has expired
            let current_time = OffsetDateTime::now_utc();

            if session.expires_at < current_time  {
                // Remove expired session
                let _ = sqlx::query!(
                    "DELETE FROM Sessions_ WHERE session_id = ?",
                    session_id
                )
                .execute(pool.get_ref())
                .await;

                info!("Session expired for session ID: {}", session_id);
                return HttpResponse::Unauthorized().json(AutoLoginResponse {
                    success: false,
                    message: "Login is needed, session expired".into(),
                    username: "".into(),
                });
            }

            // 3. If the session Id is valid, fetch the user association with the session
            let user_result = sqlx::query!(
                "SELECT user_name FROM Users_ WHERE user_id = ?",
                session.user_id
            )
            .fetch_one(pool.get_ref())
            .await;

            match user_result {
                Ok(user) => {
                    // 4. Return with session Id inside cookie
                    info!("Auto login successful for user: {}", user.user_name);
                    HttpResponse::Ok()
                        .cookie(
                            actix_web::cookie::Cookie::build("session_id", session_id.clone())
                                .http_only(true)
                                .finish(),
                        )
                        .json(AutoLoginResponse {
                            success: true,
                            message: format!("Welcome back, {}", user.user_name),
                            username: user.user_name,
                        })
                }
                Err(e) => {
                    error!("Failed to fetch user information for session ID {}: {}", session_id, e);
                    HttpResponse::InternalServerError().json(AutoLoginResponse {
                        success: false,
                        message: "Failed to fetch user information".into(),
                        username: "".into(),
                    })
                }
            }
        }
        Ok(None) => {
            info!("Invalid session ID: {}", session_id);
            HttpResponse::BadRequest().json(AutoLoginResponse {
                success: false,
                message: "Invalid session ID".into(),
                username: "".into(),
            })
        }
        Err(e) => {
            error!("Failed to validate session ID {}: {}", session_id, e);
            HttpResponse::InternalServerError().json(AutoLoginResponse {
                success: false,
                message: "Failed to validate session".into(),
                username: "".into(),
            })
        }
    }
}

pub async fn logout(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    _: web::Json<LogoutRequest>,
) -> impl Responder {
    // 1. Receive the session ID from the cookie (if not exist, return with error: "session ID does not exist")
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID does not exist in cookies for logout");
            return HttpResponse::BadRequest().json(LogoutResponse {
                success: false,
                message: "Session ID does not exist".into(),
            });
        }
    };

    info!("Received logout request with session ID: {}", session_id);

    // 2. Check whether the session exists and is valid
    let session_result = sqlx::query!(
        "SELECT expires_at FROM Sessions_ WHERE session_id = ?",
        session_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match session_result {
        Ok(Some(session)) => {
            let current_time = OffsetDateTime::now_utc();

            // 2-1. If session is expired, return failure with message: "already expired session"
            if session.expires_at < current_time {
                info!("Session already expired for session ID: {}", session_id);
                return HttpResponse::BadRequest().json(LogoutResponse {
                    success: false,
                    message: "Already expired session".into(),
                });
            }

            // 2-2. If session is not expired, delete the session and return success with message: "logout successful"
            let delete_result = sqlx::query!(
                "DELETE FROM Sessions_ WHERE session_id = ?",
                session_id
            )
            .execute(pool.get_ref())
            .await;

            match delete_result {
                Ok(_) => {
                    info!("Logout successful for session ID: {}", session_id);
                    HttpResponse::Ok().json(LogoutResponse {
                        success: true,
                        message: "Logout successful".into(),
                    })
                }
                Err(e) => {
                    error!("Failed to delete session ID {}: {}", session_id, e);
                    HttpResponse::InternalServerError().json(LogoutResponse {
                        success: false,
                        message: "Failed to logout".into(),
                    })
                }
            }
        }
        Ok(None) => {
            info!("Session not found for session ID: {}", session_id);
            HttpResponse::BadRequest().json(LogoutResponse {
                success: false,
                message: "Session not found".into(),
            })
        }
        Err(e) => {
            error!("Failed to fetch session ID {}: {}", session_id, e);
            HttpResponse::InternalServerError().json(LogoutResponse {
                success: false,
                message: "Failed to check session".into(),
            })
        }
    }
}
