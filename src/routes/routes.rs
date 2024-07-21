use actix_web::web;

pub fn login_configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api-login")
            .route("", web::get().to(super::login::login_handlers::login_get))  // Add this line
            .route("/", web::get().to(super::login::login_handlers::login_get))  // Add this line
            .route("/check-username", web::post().to(super::login::login_handlers::check_username))
            .route("/check-email", web::post().to(super::login::login_handlers::check_email))
            .route("/register", web::post().to(super::login::login_handlers::register))
            .route("/login", web::post().to(super::login::login_handlers::login))
            .route("/auto-login", web::post().to(super::login::login_handlers::auto_login))
    );
}
