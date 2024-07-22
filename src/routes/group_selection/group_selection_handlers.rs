use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use log::error;
use super::group_selection_models::{GroupListResponse, Group};

// Default handler for group selection root
pub async fn group_selection_get() -> impl Responder {
    HttpResponse::Ok().body("Hello, this is the Group Selection endpoint.")
}

// Handler to get the group list
pub async fn get_group_list(
    pool: web::Data<MySqlPool>,
    // Assuming you have a way to get the current user's ID
    current_user_id: i32,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT g.group_name, g.owner_username, ug.writeable
        FROM Groups_ g
        INNER JOIN UserGroups_ ug ON g.group_id = ug.group_id
        WHERE ug.user_id = ?
        "#,
        current_user_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(records) => {
            let groups: Vec<Group> = records.into_iter().map(|record| Group {
                group_name: record.group_name,
                writeable: record.writeable,
                owner_username: record.owner_username,
            }).collect();

            HttpResponse::Ok().json(GroupListResponse { groups })
        }
        Err(e) => {
            error!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
