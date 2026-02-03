use std::env;

use actix_web::{http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(serde::Deserialize, sqlx::Type, Debug)]
#[sqlx(type_name = "app_status", rename_all = "UPPERCASE")]
pub enum AppStatus {
    PENDING,
    RUNNING,
    STOPPED,
    FAILED,
}

#[derive(Deserialize, Debug)]
pub struct Application {
    pub name: String,
    pub command: String,
    pub status: AppStatus,
    pub port: i32,
}
