use actix_web::{web, HttpResponse, HttpRequest, Responder};
use sqlx::MySqlPool;
use log::error;
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
};

pub async fn login_get() -> impl Responder {
    HttpResponse::Ok().body("Hello this is Flow'er's Login endpoint.")
}

// Check if username is unique
pub async fn check_username(
    pool: web::Data<MySqlPool>,
    req: web::Json<CheckUsernameRequest>,
) -> impl Responder {
    let username = &req.username;
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM Users_ WHERE user_name = ?",
        username
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            let is_unique = record.count == 0;
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
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM Users_ WHERE user_email = ?",
        email
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            let is_unique = record.count == 0;
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
        Ok(_) => HttpResponse::Ok().json(RegisterResponse {
            success: true,
            message: "User registered successfully".into(),
        }),
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

    // 2. Get the user data from database with username
    let result = sqlx::query!(
        "SELECT user_id, password_hash FROM Users_ WHERE user_name = ?",
        username
    )
    .fetch_one(pool.get_ref())
    .await;

    let user = match result {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                message: "Invalid username or password".into(),
            });
        }
    };

    // 3. Validate hashed password in DB and given password
    let valid = match verify(password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            return HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                message: "Invalid username or password".into(),
            });
        }
    };

    if !valid {
        return HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            message: "Invalid username or password".into(),
        });
    }

    // 4. Check if user already has an active session
    let session_check = sqlx::query!(
        "SELECT session_id FROM Sessions_ WHERE user_id = ?",
        user.user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    if let Ok(Some(_)) = session_check {
        return HttpResponse::BadRequest().json(LoginResponse {
            success: false,
            message: "User already has an active session".into(),
        });
    }

    // 4-1 & 4-2. Create session ID with UUID and set expiration time
    let session_id = Uuid::new_v4().to_string();
    let expires_at = if req.remember_me {
        Utc::now() + Duration::days(10)
    } else {
        Utc::now() + Duration::minutes(30)
    };

    // 5. Insert into Sessions_ table with (session_id, user_id, expiration_time)
    let insert_result = sqlx::query!(
        "INSERT INTO Sessions_ (session_id, user_id, expires_at, is_persistent) VALUES (?, ?, ?, ?)",
        session_id,
        user.user_id,
        expires_at,
        req.remember_me
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        error!("Failed to insert session: {}", e);
        return HttpResponse::InternalServerError().json(LoginResponse {
            success: false,
            message: "Failed to create session".into(),
        });
    }

    // 6. Return session ID inside cookie to client
    HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("session_id", session_id.clone())
                .http_only(true)
                .finish(),
        )
        .json(LoginResponse {
            success: true,
            message: "Login successful".into(),
        })
}


/*
// Auto-login request and response
#[derive(Deserialize)]
pub struct AutoLoginRequest {
}

#[derive(Serialize)]
pub struct AutoLoginResponse {
    pub success: bool,
    pub message: String,
}
*/

/*
DATABASE SCHEMA:
CREATE TABLE Users_ (
  user_id INT AUTO_INCREMENT PRIMARY KEY,
  user_name VARCHAR(255) UNIQUE NOT NULL,
  user_email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL
);

CREATE TABLE Sessions_ (
  session_id VARCHAR(255) PRIMARY KEY,
  user_id INT UNIQUE NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  is_persistent BOOLEAN DEFAULT false,
  FOREIGN KEY (user_id) REFERENCES Users_(user_id)
);

*/
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
            return HttpResponse::Unauthorized().json(AutoLoginResponse {
                success: false,
                message: "Session ID not found in cookies".into(),
            });
        }
    };

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

                return HttpResponse::Unauthorized().json(AutoLoginResponse {
                    success: false,
                    message: "Login is needed, session expired".into(),
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
                    HttpResponse::Ok()
                        .cookie(
                            actix_web::cookie::Cookie::build("session_id", session_id.clone())
                                .http_only(true)
                                .finish(),
                        )
                        .json(AutoLoginResponse {
                            success: true,
                            message: format!("Welcome back, {}", user.user_name),
                        })
                }
                Err(_) => HttpResponse::InternalServerError().json(AutoLoginResponse {
                    success: false,
                    message: "Failed to fetch user information".into(),
                }),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(AutoLoginResponse {
            success: false,
            message: "Invalid session ID".into(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(AutoLoginResponse {
            success: false,
            message: "Failed to validate session".into(),
        }),
    }
}