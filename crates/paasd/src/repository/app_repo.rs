use crate::models::{Application, PatchApplication};
use sqlx::{Error, PgPool};

pub async fn insert_application(pool: &PgPool, app: &Application) -> Result<(), Error> {
    let query = "INSERT INTO apps (name, command, status, port) VALUES ($1, $2, $3, $4)";
    sqlx::query(query)
        .bind(&app.name)
        .bind(&app.command)
        .bind(&app.status)
        .bind(&app.port)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_applications(pool: &PgPool) -> Result<Vec<Application>, Error> {
    let apps = sqlx::query_as(r#"SELECT id, name, command, status, port FROM apps"#)
        .fetch_all(pool)
        .await?;
    Ok(apps)
}

pub async fn get_application(pool: &PgPool, app_id: i32) -> Result<Application, Error> {
    let app = sqlx::query_as(r#"SELECT id, name, command, status, port FROM apps where id = $1"#)
        .bind(app_id)
        .fetch_one(pool)
        .await?;

    Ok(app)
}

pub async fn patch_application(
    pool: &PgPool,
    app_id: i32,
    app: &PatchApplication,
) -> Result<(), Error> {
    let mut query = String::from("UPDATE apps SET ");
    let mut fields = Vec::new();

    if app.name.is_some() {
        fields.push(format!("name = ${}", fields.len() + 1));
    }

    if app.command.is_some() {
        fields.push(format!("command =${}", fields.len() + 1));
    }

    if app.status.is_some() {
        fields.push(format!("status = ${}", fields.len() + 1));
    }

    if app.port.is_some() {
        fields.push(format!("port = ${}", fields.len() + 1));
    }

    if fields.is_empty() {
        return Ok(());
    }

    query.push_str(&fields.join(", "));
    query.push_str(&format!(" WHERE id =${}", fields.len() + 1));

    let mut sql = sqlx::query(&query);

    if let Some(name) = &app.name {
        sql = sql.bind(name);
    }

    if let Some(command) = &app.command {
        sql = sql.bind(command);
    }

    if let Some(status) = &app.status {
        sql = sql.bind(status);
    }

    if let Some(port) = &app.port {
        sql = sql.bind(port);
    }

    sql = sql.bind(app_id);

    sql.execute(pool).await?;

    Ok(())
}
