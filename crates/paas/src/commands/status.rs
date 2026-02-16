use serde::Deserialize;
use shared::Application;
use std::{fmt::format, fs::read_to_string, path::Path};
use uuid::Uuid;

use anyhow::Ok;
use reqwest::Client;

#[derive(Debug, Deserialize)]
pub struct PaasConfig {
    pub id: Option<Uuid>,
}

pub async fn check_status() -> anyhow::Result<()> {
    let filename = "paas.toml";

    if !Path::new("paas.toml").exists() {
        println!("can not find paas.toml config file. run `paas init` to initialize a project.");
        return Ok(());
    }

    let config_file_content = read_to_string(filename)?;

    let app_data: PaasConfig = toml::from_str(&config_file_content)?;

    if app_data.id.is_none() {
        println!("Project not deployed.");
        println!("Run `paas deploy`");
    }

    println!(
        "getting the status of your application (id: {:?}",
        app_data.id
    );
    println!();

    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}", app_data.id.unwrap());

    let res = client.get(&url).send().await?;
    let application_infos: Application = res.json().await?;

    let app_info_to_print = format!(
        "Application: {}\nId: {:?}\nStatus: {:?}\nPort: {}\nCommand: {}",
        application_infos.name,
        application_infos.id.unwrap(),
        application_infos.status,
        application_infos.port,
        application_infos.command
    );

    println!("{}", app_info_to_print);
    Ok(())
}
