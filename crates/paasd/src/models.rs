// use serde::{Deserialize, Serialize};
// use sqlx::prelude::FromRow;

// #[derive(Deserialize, Serialize, sqlx::Type, Debug)]
// #[sqlx(type_name = "app_status", rename_all = "UPPERCASE")]
// pub enum AppStatus {
//     PENDING,
//     RUNNING,
//     STOPPED,
//     FAILED,
// }

// #[derive(Deserialize, Serialize, Debug, FromRow)]
// pub struct Application {
//     pub name: String,
//     pub command: String,
//     pub status: AppStatus,
//     pub port: i32,
// }

pub use shared::{AppStatus, Application, PatchApplication};

// #[derive(Deserialize, Debug, FromRow)]
// pub struct PatchApplication {
//     pub name: Option<String>,
//     pub command: Option<String>,
//     pub status: Option<AppStatus>,
//     pub port: Option<i32>,
// }
