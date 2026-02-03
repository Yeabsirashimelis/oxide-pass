
use actix_web::{App, HttpResponse, HttpServer, Responder, http::header::ContentType, web};
use serde::Deserialize;
use sqlx::PgPool;
use crate::models::Application;
use crate::repository::app_repo::insert_application;



pub async fn post_program(pool: web::Data<PgPool>, app: web::Json<Application>) -> impl Responder {
    println!("{:?}", app);
   
match insert_application(pool.get_ref(), &app).await {
    Ok(_)=> HttpResponse::Ok().body("Application Program Registered Successfully"),
    Err(e)=> {
        eprintln!("DB Error: {}", e);
        return HttpResponse::InternalServerError().finish();
    }
};

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("application registered successfully")
}