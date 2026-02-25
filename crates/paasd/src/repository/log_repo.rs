use sqlx::{Error, PgPool};
use uuid::Uuid;
use shared::{AppLog, NewAppLog};

pub async fn insert_log(pool: &PgPool, log: &NewAppLog) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO logs (app_id, stream, message) VALUES ($1, $2, $3)"
    )
    .bind(log.app_id)
    .bind(&log.stream)
    .bind(&log.message)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_logs(pool: &PgPool, app_id: Uuid, limit: i64) -> Result<Vec<AppLog>, Error> {
    let logs = sqlx::query_as(
        "SELECT id, app_id, stream, message, created_at FROM logs WHERE app_id = $1 ORDER BY created_at ASC LIMIT $2"
    )
    .bind(app_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

pub async fn get_logs_since(pool: &PgPool, app_id: Uuid, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<AppLog>, Error> {
    let logs = sqlx::query_as(
        "SELECT id, app_id, stream, message, created_at FROM logs WHERE app_id = $1 AND created_at > $2 ORDER BY created_at ASC"
    )
    .bind(app_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}
