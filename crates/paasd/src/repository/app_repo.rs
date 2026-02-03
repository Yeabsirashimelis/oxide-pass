use sqlx::{PgPool, Error};
use crate::models::Application;


pub async fn insert_application(pool: &PgPool, app: &Application) -> Result<(), Error> {
    let query = "INSERT INTO App (name, command, status, port) VALUES ($1, $2, $3, $4)";
    sqlx::query(query)
        .bind(&app.name)
        .bind(&app.command)
        .bind(&app.status)
        .bind(&app.port)
        .execute(pool)
        .await?;

  Ok(())
}