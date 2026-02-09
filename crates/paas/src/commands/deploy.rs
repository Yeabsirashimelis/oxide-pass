use std::{
    collections::HashMap,
    fs::File,
    io::{self},
    path::Path,
};

use reqwest::Client;
use serde::Deserialize;
use shared::Application;

use crate::commands::status;

#[derive(Debug, Deserialize)]
pub struct PaasConfig {
    pub name: String,
    pub runtime: String,
    pub command: String,
    pub port: Option<i32>,
}

pub async fn deploy_project() -> anyhow::Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        println!("Initialize the project first. use 'paas -- init' for that.");
        return Ok(());
    }

    let file = File::open(filename)?;

    // wrap the file in a bufreader for efficient line-by-line reading
    let reader = io::BufReader::new(file);

    //read the wholefile into string
    let content = std::fs::read_to_string(filename)?;

    //map/ deserialize it directly into out struct
    let app_data: PaasConfig = toml::from_str(&content)?;

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
        println!("Project Successfully deployed");
    } else {
        eprintln!("Deployment failed with status: {}", res.status());
    }

    Ok(())
}
