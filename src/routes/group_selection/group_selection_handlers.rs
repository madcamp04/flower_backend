use actix_web::{web, HttpResponse, HttpRequest, Responder};
use sqlx::MySqlPool;
use log::{error, info};
use super::group_selection_models::{GetGroupListRequest, GetGroupListResponse, Group};

// Default handler for group selection root
pub async fn group_selection_get() -> impl Responder {
    HttpResponse::Ok().body("Hello, this is the Group Selection endpoint.")
}

// Handler to get the group list
pub async fn get_group_list(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    _: web::Json<GetGroupListRequest>,
) -> impl Responder {
    // Initialize an empty group list response
    let mut response = GetGroupListResponse {
        groups: Vec::new(),
    };

    // Extract session ID from the cookie
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Session ID not found in cookies for get_group_list");
            return HttpResponse::BadRequest().json(response); // Return empty response in case of missing session ID
        }
    };

    info!("Received request to get group list with session ID: {}", session_id);

    // Verify session ID and get user information
    let session_result = sqlx::query!(
        "SELECT user_id FROM Sessions_ WHERE session_id = ? AND expires_at > NOW()",
        session_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let user_id = match session_result {
        Ok(session) => session.user_id,
        Err(_) => {
            info!("Invalid or expired session ID: {}", session_id);
            return HttpResponse::BadRequest().json(response); // Return empty response in case of invalid session ID
        }
    };

    // Fetch the groups and their details in a single query
    let groups_result = sqlx::query!(
        "SELECT g.group_name, u.user_name AS owner_username, gum.writeable 
         FROM GroupUserMapping_ gum
         JOIN Groups_ g ON gum.group_id = g.group_id
         JOIN Users_ u ON g.owner_user_id = u.user_id
         WHERE gum.user_id = ?",
        user_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match groups_result {
        Ok(records) => {
            response.groups = records.into_iter().map(|record| Group {
                group_name: record.group_name,
                writeable: record.writeable.unwrap_or(0) != 0, // Convert Option<i8> to bool
                owner_username: record.owner_username,
            }).collect();

            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            error!("Failed to fetch group list for user_id {}: {}", user_id, e);
            HttpResponse::InternalServerError().json(response) // Return empty response in case of a database error
        }
    }
}

