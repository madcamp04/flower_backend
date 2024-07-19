use actix_web::{web, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use dotenv::dotenv;

mod routes;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::init)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
