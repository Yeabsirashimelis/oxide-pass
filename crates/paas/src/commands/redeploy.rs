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

    println!("Redeploying app with id: {}", app_id);

    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/redeploy", app_id);

    let res = client.post(&url).send().await?;

    if res.status().is_success() {
        println!("Application successfully redeployed.");
    } else {
        eprintln!("Redeploy failed with status: {}", res.status());
    }

    Ok(())
}
