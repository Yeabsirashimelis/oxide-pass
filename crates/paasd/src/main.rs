mod handlers;
mod models;
mod repository;

use std::env;

use crate::handlers::app_handlers::{get_programs, post_program};
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;

//db connection
async fn connect_db() -> Result<sqlx::PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    Ok(pool)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //load the environmnet variables at the start of the server
    dotenvy::dotenv().ok();

    let addr = ("127.0.0.1", 8080);
    let pool = connect_db().await.expect("DB connection failed");
    println!("app is bound to http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::post().to(post_program))
            .route("/", web::get().to(get_programs))
    })
    .bind(addr)?
    .run()
    .await
}
