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
        let mut last_count = 0usize;
        loop {
            match fetch_logs(&app_id, 200).await {
                Result::Ok(logs) => {
                    if logs.len() > last_count {
                        for log in &logs[last_count..] {
                            print_log(log);
                        }
                        last_count = logs.len();
                    }
                }
                Err(e) => eprintln!("Error fetching logs: {}", e),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
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

fn print_log(log: &AppLog) {
    let prefix = if log.stream == "stderr" {
        "[ERR]"
    } else {
        "[OUT]"
    };
    println!("{} {} {}", log.created_at, prefix, log.message);
}
