use crate::models::Application;
use crate::repository::app_repo::{get_applications, insert_application};
use actix_web::{HttpResponse, Responder, http::header::ContentType, web};
use sqlx::PgPool;

pub async fn post_program(pool: web::Data<PgPool>, app: web::Json<Application>) -> impl Responder {
    println!("{:?}", app);

    match insert_application(pool.get_ref(), &app).await {
        Ok(_) => HttpResponse::Ok().body("Application Program Registered Successfully"),
        Err(e) => {
            eprintln!("DB Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("application registered successfully")
}

pub async fn get_programs(pool: web::Data<PgPool>) -> impl Responder {
    match get_applications(pool.get_ref()).await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => {
            eprintln!("DB Error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }
}
