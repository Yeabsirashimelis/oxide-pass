use std::path::Path;

use reqwest::Client;
use uuid::Uuid;

pub async fn redeploy_project() -> anyhow::Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        println!("Initialize the project first. use 'paas init' for that.");
        return Ok(());
    }

    let content = std::fs::read_to_string(filename)?;
    let app_data: toml::Value = toml::from_str(&content)?;

    let id = match app_data.get("id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            println!("Project not deployed yet. Use `paas deploy` first.");
            return Ok(());
        }
    };

    let app_id: Uuid = id.parse()?;
    let port = app_data.get("port").and_then(|v| v.as_integer()).unwrap_or(3000) as i32;

    println!("Redeploying app with id: {}", app_id);

    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/redeploy", app_id);

    let res = client.post(&url).json(&serde_json::json!({ "port": port })).send().await?;

    if res.status().is_success() {
        println!("Application successfully redeployed.");
        println!("Starting application...");

        // Wait for app to start and detect actual port
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let client2 = Client::new();
        let status_url = format!("http://127.0.0.1:8080/apps/{}/status", app_id);
        if let Ok(status_res) = client2.get(&status_url).send().await {
            if let Ok(status_body) = status_res.json::<serde_json::Value>().await {
                let port = status_body["port"].as_i64().unwrap_or(3000);
                println!("Application is running on port {}", port);
                println!("Local: http://localhost:{}", port);
            }
        }
    } else {
        eprintln!("Redeploy failed with status: {}", res.status());
    }

    Ok(())
}
