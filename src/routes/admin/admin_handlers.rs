use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use log::error;
use super::admin_models::AdminDefaultResponse;

pub async fn session_reset(
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    // Attempt to delete all sessions from the Sessions_ table
    let result = sqlx::query!(
        "DELETE FROM Sessions_"
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(AdminDefaultResponse{
            success: true,
            message: "All sessions have been reset successfully".into(),
        }),
        Err(e) => {
            error!("Failed to reset sessions: {}", e);
            HttpResponse::InternalServerError().json(AdminDefaultResponse{
                success: false,
                message: "Failed to reset sessions".into(),
            })
        }
    }
}
