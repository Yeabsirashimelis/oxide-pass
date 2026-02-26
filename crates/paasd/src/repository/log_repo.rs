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

/// Delete logs older than 7 days
pub async fn delete_old_logs(pool: &PgPool) -> Result<u64, Error> {
    let result = sqlx::query(
        "DELETE FROM logs WHERE created_at < NOW() - INTERVAL '7 days'"
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Keep only the last 1000 log entries per app, delete the rest
pub async fn trim_logs_per_app(pool: &PgPool) -> Result<u64, Error> {
    let result = sqlx::query(
        "DELETE FROM logs WHERE id IN (
            SELECT id FROM (
                SELECT id, ROW_NUMBER() OVER (PARTITION BY app_id ORDER BY created_at DESC) as rn
                FROM logs
            ) ranked
            WHERE rn > 1000
        )"
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Run all cleanup tasks
pub async fn cleanup_logs(pool: &PgPool) {
    match delete_old_logs(pool).await {
        Ok(n) => println!("Log cleanup: deleted {} logs older than 7 days", n),
        Err(e) => eprintln!("Log cleanup error (old logs): {}", e),
    }
    match trim_logs_per_app(pool).await {
        Ok(n) => println!("Log cleanup: trimmed {} excess logs (keeping last 1000 per app)", n),
        Err(e) => eprintln!("Log cleanup error (trim): {}", e),
    }
}
