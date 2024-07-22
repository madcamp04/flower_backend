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
    );
}