use std::{
    fs::{self},
    io::Write,
    path::Path,
};

use anyhow::Ok;
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
        println!("Project already deployed (id: {})", existing_id);
        println!("Use `paas redeploy` to restart/update.");
        return Ok(());
    }

    println!("Deploying: {} using {}", app_data.name, app_data.command);

    let request_payload = Application {
        name: app_data.name,
        command: app_data.command,
        port: app_data.port.unwrap_or(3000),
        status: shared::AppStatus::PENDING,
        id: None,
    };

    let client = Client::new();
    let url = "http://127.0.0.1:8080/apps";

    let res = client.post(url).json(&request_payload).send().await?;

    if res.status().is_success() {
        let application_id: Uuid = res.json().await?;

        let mut file = fs::OpenOptions::new().append(true).open("paas.toml")?;
        writeln!(file, "id = \"{}\"", application_id);

        println!("Project Successfully deployed");
    } else {
        eprintln!("Deployment failed with status: {}", res.status());
    }

    Ok(())
}
