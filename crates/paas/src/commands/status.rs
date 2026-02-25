use serde::Deserialize;
use std::{fs::read_to_string, path::Path};
use uuid::Uuid;

use anyhow::Ok;
use reqwest::Client;

#[derive(Debug, Deserialize)]
pub struct PaasConfig {
    pub id: Option<Uuid>,
}

pub async fn check_status() -> anyhow::Result<()> {
    let filename = "paas.toml";

    if !Path::new(filename).exists() {
        println!("can not find paas.toml config file. run `paas init` to initialize a project.");
        return Ok(());
    }

    let config_file_content = read_to_string(filename)?;

    let app_data: PaasConfig = toml::from_str(&config_file_content)?;

    if app_data.id.is_none() {
        println!("Project not deployed.");
        println!("Run `paas deploy`");
        return Ok(());
    }

    println!(
        "getting the status of your application (id: {:?}",
        app_data.id
    );
    println!();

    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/status", app_data.id.unwrap());

    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        eprintln!("Failed to fetch status: {}", res.status());
        return Ok(());
    }

    let info: serde_json::Value = res.json().await?;

    println!(
        "Application: {}\nId: {}\nStatus: {}\nPID: {}\nPort: {}\nCommand: {}",
        info["name"].as_str().unwrap_or("unknown"),
        info["id"].as_str().unwrap_or("unknown"),
        info["status"].as_str().unwrap_or("unknown"),
        info["pid"].as_i64().map(|p: i64| p.to_string()).unwrap_or("none".into()),
        info["port"].as_i64().unwrap_or(0),
        info["command"].as_str().unwrap_or("unknown"),
    );

    Ok(())
}
