use crate::models::{Application, PatchApplication};
use crate::repository::app_repo::{
    get_application, get_applications, insert_application, patch_application,
};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;

pub async fn post_program(pool: web::Data<PgPool>, app: web::Json<Application>) -> impl Responder {
    println!("{:?}", app);

    match insert_application(pool.get_ref(), &app).await {
        Ok(app_id) => HttpResponse::Ok().json(app_id),
        Err(error) => {
            eprintln!("DB Error: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub async fn get_programs(pool: web::Data<PgPool>) -> impl Responder {
    match get_applications(pool.get_ref()).await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(error) => {
            eprintln!("DB Error: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub async fn get_program(pool: web::Data<PgPool>, path: web::Path<i32>) -> impl Responder {
    let app_id = path.into_inner();
    match get_application(pool.get_ref(), app_id).await {
        Ok(app) => HttpResponse::Ok().json(app),
        Err(error) => match error {
            sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
            _ => {
                eprintln!("DB Error: {}", error);
                return HttpResponse::InternalServerError().finish();
            }
        },
    }
}

pub async fn patch_program(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    edited_app_info: web::Json<PatchApplication>,
) -> impl Responder {
    let app_id = path.into_inner();
    match patch_application(pool.get_ref(), app_id, &edited_app_info).await {
        Ok(_) => HttpResponse::Ok().body(format!(
            "Application Program ID = {} Information Successfully Updated",
            app_id
        )),
        Err(error) => {
            eprintln!("DB Error: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}
