use std::{
    fs::{self},
    io::Write,
    path::Path,
};

use reqwest::Client;
use serde::Deserialize;
use shared::Application;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaasConfig {
    pub name: String,
    pub runtime: String,
    pub command: String,
    pub port: Option<i32>,
    pub id: Option<Uuid>,
}

pub async fn deploy_project() -> anyhow::Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        println!("Initialize the project first. use 'paas init' for that.");
        return Ok(());
    }

    // let _file = File::open(filename)?;

    //read the wholefile into string
    let content = std::fs::read_to_string(filename)?;

    //map/ deserialize it directly into out struct
    let app_data: PaasConfig = toml::from_str(&content)?;

    if let Some(existing_id) = app_data.id {
        println!("Project already deployed (id: {}).", existing_id);
        println!("  - To restart it, use `paas redeploy`.");
        println!("  - To stop it, use `paas stop`.");
        println!("  - To deploy as a brand new app, remove the `id` line from paas.toml.");
        return Ok(());
    }

    println!("Deploying: {} using {}", app_data.name, app_data.command);

    let current_dir = std::env::current_dir()?
        .to_string_lossy()
        .to_string();

    let request_payload = Application {
        name: app_data.name,
        command: app_data.command,
        port: app_data.port.unwrap_or(3000),
        status: shared::AppStatus::PENDING,
        id: None,
        working_dir: current_dir,
        pid: None,
    };

    let client = Client::new();
    let url = "http://127.0.0.1:8080/apps";

    let res = client.post(url).json(&request_payload).send().await?;

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await?;
        let application_id: Uuid = body["id"].as_str().unwrap_or_default().parse()?;

        let mut file = fs::OpenOptions::new().append(true).open("paas.toml")?;
        writeln!(file, "\nid = \"{}\"", application_id)?;

        println!("Project Successfully deployed");
        println!("Starting application...");

        // Wait for app to start and detect actual port
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let client2 = Client::new();
        let status_url = format!("http://127.0.0.1:8080/apps/{}/status", application_id);
        if let Ok(status_res) = client2.get(&status_url).send().await {
            if let Ok(status_body) = status_res.json::<serde_json::Value>().await {
                let port = status_body["port"].as_i64().unwrap_or(3000);
                println!("Application is running on port {}", port);
                println!("Local: http://localhost:{}", port);
            }
        }
    } else if res.status() == reqwest::StatusCode::CONFLICT {
        let body = res.text().await.unwrap_or_default();
        eprintln!("Deployment failed: {}", body);
        eprintln!("Tip: Change the port in paas.toml and try again.");
    } else {
        eprintln!("Deployment failed with status: {}", res.status());
    }

    Ok(())
}
