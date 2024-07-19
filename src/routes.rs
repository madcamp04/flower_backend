use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::{
    user::User,
    // project_manager::ProjectManager,
    // worker::Worker,
    // pm_worker_mapping::PMWorkerMapping,
    // tag::Tag,
    // project::Project,
    // tag_project_mapping::TagProjectMapping,
    // task::Task,
    // dependency::Dependency,
};

#[get("/users")]
async fn get_users(pool: web::Data<MySqlPool>) -> impl Responder {
    let users = sqlx::query_as!(User, "SELECT * FROM Users")
        .fetch_all(pool.get_ref())
        .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users);
}
