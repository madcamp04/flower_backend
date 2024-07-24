// use std::path;

use actix_web::web;

pub fn admin_configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/delete/all/the/sessions/BECAREFUL", web::get().to(super::admin::admin_handlers::session_reset))
    );
}

use super::login::login_handlers;

pub fn login_configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api-login")
            .route("", web::get().to(login_handlers::login_get))  // Add this line
            .route("/", web::get().to(login_handlers::login_get))  // Add this line
            .route("/check-username", web::post().to(login_handlers::check_username))
            .route("/check-email", web::post().to(login_handlers::check_email))
            .route("/register", web::post().to(login_handlers::register))
            .route("/login", web::post().to(login_handlers::login))
            .route("/auto-login", web::post().to(login_handlers::auto_login))
            .route("/logout", web::post().to(login_handlers::logout))
    );
}

use super::group_selection::group_selection_handlers;

pub fn group_selection_configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api-group-selection")
            .route("", web::get().to(group_selection_handlers::group_selection_get))  // Add this line
            .route("/", web::get().to(group_selection_handlers::group_selection_get))  // Add this line
            .route("/group-list", web::post().to(group_selection_handlers::get_group_list))
            .route("/add-group", web::post().to(group_selection_handlers::add_group))
            .route("/update-group", web::post().to(group_selection_handlers::update_group))
    );
}

use super::group_view::group_view_handlers;

pub fn group_view_configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api-group-view")
            .route("", web::get().to(group_view_handlers::group_view_get))  // Add this line
            .route("/", web::get().to(group_view_handlers::group_view_get))  // Add this line
            .route("/worker-list", web::post().to(group_view_handlers::get_worker_list))
            .route("/add-worker", web::post().to(group_view_handlers::add_worker))
            .route("/tag-list", web::post().to(group_view_handlers::get_tag_list))
            .route("/add-tag", web::post().to(group_view_handlers::add_tag))
            .route("/update-tag", web::post().to(group_view_handlers::update_tag))
            .route("/task-list/by-tag-list", web::post().to(group_view_handlers::get_task_list_by_tag_list))
            .route("/task-list/by-project-name", web::post().to(group_view_handlers::get_task_list_by_project_name))
            .route("/project-list", web::post().to(group_view_handlers::get_project_list))
    );
}

use super::project_view::project_view_handlers;

pub fn project_view_configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api-project-view")
            .route("", web::get().to(project_view_handlers::project_view_get))  // Add this line
            .route("/", web::get().to(project_view_handlers::project_view_get))  // Add this line
            .route("/project-detail", web::post().to(project_view_handlers::get_project_detail))
            .route("/add-project", web::post().to(project_view_handlers::add_project))
            // .route("/update-project", web::post().to(project_view_handlers::update_project))
            .route("/task-detail", web::post().to(project_view_handlers::get_task_detail))
            .route("/add-task", web::post().to(project_view_handlers::add_task))
            // .route("/update-task", web::post().to(project_view_handlers::update_task))
    );
}