use std::path::Path;

use anyhow::Ok;
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct PaasConfig {
    pub id: Option<Uuid>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppLog {
    pub stream: String,
    pub message: String,
    pub created_at: String,
}

pub async fn show_logs(follow: bool) -> anyhow::Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        println!("Initialize the project first. Use 'paas init' for that.");
        return Ok(());
    }

    let content = std::fs::read_to_string(filename)?;
    let app_data: PaasConfig = toml::from_str(&content)?;

    let app_id = match app_data.id {
        Some(id) => id,
        None => {
            println!("App not deployed yet. Use 'paas deploy' first.");
            return Ok(());
        }
    };

    let app_name = app_data.name.unwrap_or_else(|| "app".to_string());
    println!("Fetching logs for {}...", app_name);

    if follow {
        // Start by fetching existing logs
        let initial_logs = fetch_logs(&app_id, 100).await.unwrap_or_default();
        let mut last_timestamp = if initial_logs.is_empty() {
            chrono::Utc::now().to_rfc3339()
        } else {
            for log in &initial_logs {
                print_log(log);
            }
            initial_logs.last().unwrap().created_at.clone()
        };

        // Poll for new logs using since timestamp
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            match fetch_logs_since(&app_id, &last_timestamp).await {
                Result::Ok(logs) => {
                    if !logs.is_empty() {
                        for log in &logs {
                            print_log(log);
                        }
                        last_timestamp = logs.last().unwrap().created_at.clone();
                    }
                }
                Err(e) => eprintln!("Error fetching logs: {}", e),
            }
        }
    } else {
        match fetch_logs(&app_id, 100).await {
            Result::Ok(logs) => {
                if logs.is_empty() {
                    println!("No logs yet.");
                } else {
                    for log in &logs {
                        print_log(log);
                    }
                }
            }
            Err(e) => eprintln!("Error fetching logs: {}", e),
        }
    }

    Ok(())
}

async fn fetch_logs(app_id: &Uuid, limit: i64) -> anyhow::Result<Vec<AppLog>> {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/logs?limit={}", app_id, limit);
    let res = client.get(&url).send().await?;
    let logs: Vec<AppLog> = res.json().await?;
    Ok(logs)
}

async fn fetch_logs_since(app_id: &Uuid, since: &str) -> anyhow::Result<Vec<AppLog>> {
    let client = Client::new();
    let since_encoded = urlencoding::encode(since);
    let url = format!("http://127.0.0.1:8080/apps/{}/logs?since={}", app_id, since_encoded);
    let res = client.get(&url).send().await?;
    let logs: Vec<AppLog> = res.json().await?;
    Ok(logs)
}

fn print_log(log: &AppLog) {
    let prefix = if log.stream == "stderr" {
        "[ERR]"
    } else {
        "[OUT]"
    };
    println!("{} {} {}", log.created_at, prefix, log.message);
}
