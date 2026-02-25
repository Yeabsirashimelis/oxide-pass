use crate::models::{Application, PatchApplication};
use sqlx::{Error, PgPool, Row};
use uuid::Uuid;

pub async fn is_port_in_use(pool: &PgPool, port: i32) -> Result<bool, Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM apps WHERE port = $1 AND status != 'STOPPED'::app_status")
            .bind(port)
            .fetch_one(pool)
            .await?;
    Ok(row.0 > 0)
}

pub async fn insert_application(pool: &PgPool, app: &Application) -> Result<Uuid, Error> {
    let query =
        "INSERT INTO apps (name, command, status, port, working_dir) VALUES ($1, $2, $3, $4, $5) RETURNING id";

    let row = sqlx::query(query)
        .bind(&app.name)
        .bind(&app.command)
        .bind(&app.status)
        .bind(&app.port)
        .bind(&app.working_dir)
        .fetch_one(pool)
        .await?;

    Ok(row.get("id"))
}

pub async fn get_applications(pool: &PgPool) -> Result<Vec<Application>, Error> {
    let apps = sqlx::query_as(r#"SELECT id, name, command, status, port, working_dir, pid FROM apps"#)
        .fetch_all(pool)
        .await?;
    Ok(apps)
}

pub async fn get_application(pool: &PgPool, app_id: Uuid) -> Result<Application, Error> {
    let app = sqlx::query_as(r#"SELECT id, name, command, status, port, working_dir, pid FROM apps where id = $1"#)
        .bind(app_id)
        .fetch_one(pool)
        .await?;

    Ok(app)
}

pub async fn clear_pid(pool: &PgPool, app_id: Uuid) -> Result<(), Error> {
    sqlx::query("UPDATE apps SET pid = NULL WHERE id = $1")
        .bind(app_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn patch_application(
    pool: &PgPool,
    app_id: Uuid,
    app: &PatchApplication,
) -> Result<(), Error> {
    let mut query = String::from("UPDATE apps SET ");
    let mut fields = Vec::new();

    if app.name.is_some() {
        fields.push(format!("name = ${}", fields.len() + 1));
    }

    if app.command.is_some() {
        fields.push(format!("command = ${}", fields.len() + 1));
    }

    if app.status.is_some() {
        fields.push(format!("status = ${}", fields.len() + 1));
    }

    if app.port.is_some() {
        fields.push(format!("port = ${}", fields.len() + 1));
    }

    if app.working_dir.is_some() {
        fields.push(format!("working_dir = ${}", fields.len() + 1));
    }

    if app.pid.is_some() {
        fields.push(format!("pid = ${}", fields.len() + 1));
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

    if let Some(working_dir) = &app.working_dir {
        sql = sql.bind(working_dir);
    }

    if let Some(pid) = &app.pid {
        sql = sql.bind(pid);
    }

    sql = sql.bind(app_id);

    sql.execute(pool).await?;

    Ok(())
}
