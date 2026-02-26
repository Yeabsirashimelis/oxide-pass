use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "app_status", rename_all = "UPPERCASE")]
pub enum AppStatus {
    PENDING,
    RUNNING,
    STOPPED,
    FAILED,
    CRASHED,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Application {
    pub id: Option<Uuid>,
    pub name: String,
    pub command: String,
    pub status: AppStatus,
    pub port: i32,
    pub working_dir: String,
    pub pid: Option<i32>,
    pub env_vars: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct PatchApplication {
    pub name: Option<String>,
    pub command: Option<String>,
    pub status: Option<AppStatus>,
    pub port: Option<i32>,
    pub working_dir: Option<String>,
    pub pid: Option<i32>,
    pub env_vars: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct AppLog {
    pub id: i64,
    pub app_id: Uuid,
    pub stream: String,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewAppLog {
    pub app_id: Uuid,
    pub stream: String,
    pub message: String,
}
