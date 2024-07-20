use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/login")
            .route("/check-username", web::post().to(super::login_handlers::check_username))
            .route("/check-email", web::post().to(super::login_handlers::check_email))
            .route("/register", web::post().to(super::login_handlers::register))
            .route("/", web::post().to(super::login_handlers::login))
            .route("/auto-login", web::post().to(super::login_handlers::auto_login))
    );
}
