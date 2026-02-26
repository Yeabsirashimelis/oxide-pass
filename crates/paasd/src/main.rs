mod handlers;
mod models;
mod repository;

use std::env;

use crate::handlers::app_handlers::{delete_program, get_live_status, get_program, get_programs, patch_program, post_program, redeploy_program};
use crate::handlers::log_handlers::{get_app_logs, post_log};
use crate::repository::app_repo::mark_stale_apps_stopped;
use crate::repository::log_repo::cleanup_logs;
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

    // On startup, mark any apps that were left in RUNNING/PENDING state as STOPPED
    match mark_stale_apps_stopped(&pool).await {
        Ok(_) => println!("Startup: cleaned up stale app records"),
        Err(e) => eprintln!("Startup: failed to clean stale apps: {}", e),
    }

    // Run log cleanup on startup
    cleanup_logs(&pool).await;

    // Schedule log cleanup every 24 hours
    let pool_cleanup = pool.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(24 * 60 * 60)).await;
            cleanup_logs(&pool_cleanup).await;
        }
    });

    println!("app is bound to http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/apps", web::post().to(post_program))
            .route("/apps", web::get().to(get_programs))
            .route("/apps/{app_id}", web::get().to(get_program))
            .route("/apps/{app_id}", web::delete().to(delete_program))
            .route("/apps/{app_id}/status", web::get().to(get_live_status))
            .route("/apps/{app_id}", web::patch().to(patch_program))
            .route("/apps/{app_id}/redeploy", web::post().to(redeploy_program))
            .route("/apps/{app_id}/logs", web::post().to(post_log))
            .route("/apps/{app_id}/logs", web::get().to(get_app_logs))
    })
    .bind(addr)?
    .run()
    .await
}
