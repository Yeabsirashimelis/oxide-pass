use crate::models::{Application, PatchApplication};
use crate::repository::app_repo::{
    get_application, get_applications, insert_application, patch_application,
};
use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn post_program(pool: web::Data<PgPool>, app: web::Json<Application>) -> impl Responder {
    println!("{:?}", app);

    match insert_application(pool.get_ref(), &app).await {
        Ok(app_id) => {
            println!("Application saved. Starting agent...");

            let client = Client::new();

            let agent_url = "http://127.0.0.1:8001/run";

            // Send full application data to agent
            let mut app_with_id = app.into_inner();
            app_with_id.id = Some(app_id);

            let agent_response = client.post(agent_url).json(&app_with_id).send().await;

            match agent_response {
                Ok(res) if res.status().is_success() => {
                    println!("Agent started application.");
                    HttpResponse::Ok().json(app_id)
                }

                Ok(res) => {
                    eprintln!("Agent error: {}", res.status());
                    HttpResponse::InternalServerError()
                        .body("Application saved but failed to start")
                }

                Err(_) => {
                    eprintln!("Cannot reach agent.");
                    HttpResponse::InternalServerError()
                        .body("Application saved but agent unavailable")
                }
            }
        }
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

pub async fn get_program(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let app_id = path.into_inner();
    println!("app id: {}", app_id);
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
    path: web::Path<Uuid>,
    edited_app_info: web::Json<PatchApplication>,
) -> impl Responder {
    let app_id = path.into_inner();
    println!("patch app id: {}", app_id);

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
