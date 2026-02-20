use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, sqlx::Type, Debug)]
#[sqlx(type_name = "app_status", rename_all = "UPPERCASE")]
pub enum AppStatus {
    PENDING,
    RUNNING,
    STOPPED,
    FAILED,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Application {
    pub id: Option<Uuid>,
    pub name: String,
    pub command: String,
    pub status: AppStatus,
    pub port: i32,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct PatchApplication {
    pub name: Option<String>,
    pub command: Option<String>,
    pub status: Option<AppStatus>,
    pub port: Option<i32>,
}
