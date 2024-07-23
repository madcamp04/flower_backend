use actix_web::{web, App, HttpResponse, HttpServer, middleware::Logger};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use dotenv::dotenv;
// use log::info;
use env_logger::Env;

mod models;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let server_address = "0.0.0.0:8080";
    println!("Server running at http://{}", server_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("Hello, world!") }))
            .configure(routes::routes::admin_configure)
            .configure(routes::routes::login_configure) // Add this line
            .configure(routes::routes::group_selection_configure)
            .configure(routes::routes::group_view_configure)
    })
    .bind(server_address)?
    .run()
    .await
}
