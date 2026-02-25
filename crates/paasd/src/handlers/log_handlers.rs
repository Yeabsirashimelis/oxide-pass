use crate::repository::log_repo::{get_logs, get_logs_since, insert_log};
use actix_web::{HttpResponse, Responder, web};
use chrono::{DateTime, Utc};
use shared::NewAppLog;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn post_log(
    pool: web::Data<PgPool>,
    log: web::Json<NewAppLog>,
) -> impl Responder {
    match insert_log(pool.get_ref(), &log).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("DB Error inserting log: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_app_logs(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<LogQuery>,
) -> impl Responder {
    let app_id = path.into_inner();

    if let Some(since) = &query.since {
        match since.parse::<DateTime<Utc>>() {
            Ok(since_dt) => match get_logs_since(pool.get_ref(), app_id, since_dt).await {
                Ok(logs) => return HttpResponse::Ok().json(logs),
                Err(e) => {
                    eprintln!("DB Error fetching logs since: {}", e);
                    return HttpResponse::InternalServerError().finish();
                }
            },
            Err(_) => return HttpResponse::BadRequest().body("Invalid since timestamp"),
        }
    }

    let limit = query.limit.unwrap_or(100);
    match get_logs(pool.get_ref(), app_id, limit).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            eprintln!("DB Error fetching logs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct LogQuery {
    pub limit: Option<i64>,
    pub since: Option<String>,
}
