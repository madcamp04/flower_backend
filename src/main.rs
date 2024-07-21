use actix_web::{web, App, HttpResponse, HttpServer};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use dotenv::dotenv;

mod models;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let server_address = "127.0.0.1:8080";
    println!("Server running at http://{}", server_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("Hello, world!") }))
            .configure(routes::routes::login_configure) // Add this line
    })
    .bind(server_address)?
    .run()
    .await
}