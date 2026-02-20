use std::{fs::read_to_string, path::Path};

use anyhow::Ok;
use reqwest::{Client, StatusCode};
use shared::{AppStatus, Application, PatchApplication};

use crate::commands::deploy::PaasConfig;

pub async fn stop_application() -> anyhow::Result<()> {
    let filename = "paas.toml";

    if !Path::new(filename).exists() {
        println!("can not find paas.toml config file. run `paas init` to initialize a project.");
        return Ok(());
    }

    let config_file_content = read_to_string(filename)?;

    let app_data: PaasConfig = toml::from_str(&config_file_content)?;

    if app_data.id.is_none() {
        println!("Project not deployed");
        println!("Run `paas deploy`");
        return Ok(());
    }

    let app_id = app_data.id.unwrap();

    println!("Fetching application from server");

    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}", app_id);

    let res = match client.get(&url).send().await {
        Result::Ok(res) => res,
        Result::Err(_) => {
            eprintln!("Cannot connect to server");
            return Ok(());
        }
    };

    match res.status() {
        StatusCode::NOT_FOUND => {
            eprintln!("Application not found on server");
            return Ok(());
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            eprintln!("Server Error");
            return Ok(());
        }
        s if !s.is_success() => {
            eprintln!("Failed to fetch status: {}", res.status());
            return Ok(());
        }
        _ => {}
    }

    let application_infos: Application = res.json().await?;

    match application_infos.status {
        AppStatus::STOPPED => {
            println!("Application already stopped.");
            return Ok(());
        }
        AppStatus::RUNNING => {
            println!("Stopping application...");
            let url = format!("http://127.0.0.1:8080/apps/{}", app_id);

            let request_payload = PatchApplication {
                status: Option::Some(AppStatus::STOPPED),
                name: Option::None,
                command: Option::None,
                port: Option::None,
            };

            let res = match client.patch(&url).json(&request_payload).send().await {
                Result::Ok(res) => res,
                Result::Err(_) => {
                    eprintln!("Cannot connect to server");
                    return Ok(());
                }
            };

            match res.status() {
                StatusCode::OK => {
                    println!("Application stopped successfully.")
                }
                StatusCode::NOT_FOUND => {
                    eprintln!("Application not found on server.");
                }
                StatusCode::INTERNAL_SERVER_ERROR => {
                    eprintln!("Server error.");
                }
                s => {
                    eprintln!(" Failed to stop application: {}", s);
                }
            }
        }
        _ => {
            println!("Application is not running.");
            return Ok(());
        }
    }

    Ok(())
}
