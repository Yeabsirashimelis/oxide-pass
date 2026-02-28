use crate::models::{Application, AppStatus, PatchApplication};
use crate::repository::app_repo::{
    clear_pid, delete_application, get_application, get_applications, insert_application,
    is_port_in_use, patch_application,
};
use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

async fn kill_app(pid: i32) {
    let client = Client::new();
    let body = serde_json::json!({ "pid": pid });
    if let Err(e) = client
        .post("http://127.0.0.1:8001/stop")
        .json(&body)
        .send()
        .await
    {
        eprintln!("Failed to kill process {}: {}", pid, e);
    }
}

async fn start_app(app: Application) {
    let client = Client::new();
    if let Err(e) = client
        .post("http://127.0.0.1:8001/run")
        .json(&app)
        .send()
        .await
    {
        eprintln!("Failed to start app: {}", e);
    }
}

pub async fn post_program(pool: web::Data<PgPool>, app: web::Json<Application>) -> impl Responder {
    println!("{:?}", app);

    // Check for port conflicts
    match is_port_in_use(pool.get_ref(), app.port).await {
        Ok(true) => {
            eprintln!("Port {} is already in use by another running app", app.port);
            return HttpResponse::Conflict().body(format!(
                "Port {} is already in use by another running application",
                app.port
            ));
        }
        Err(e) => {
            eprintln!("DB Error checking port: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(false) => {} // Port is free, continue
    }

    match insert_application(pool.get_ref(), &app).await {
        Ok(app_id) => {
            println!("Application saved. Starting agent...");

            let client = Client::new();

            let agent_url = "http://127.0.0.1:8001/run";

            // Send full application data to agent
            let mut app_with_id = app.into_inner();
            app_with_id.id = Some(app_id);

            let agent_response = client.post(agent_url).json(&app_with_id).send().await;

            match agent_response {
                Ok(res) if res.status().is_success() => {
                    println!("Agent started application.");
                    HttpResponse::Ok().json(serde_json::json!({
                        "id": app_id,
                        "port": app_with_id.port,
                    }))
                }

                Ok(res) => {
                    eprintln!("Agent error: {}", res.status());
                    HttpResponse::InternalServerError()
                        .body("Application saved but failed to start")
                }

                Err(_) => {
                    eprintln!("Cannot reach agent.");
                    HttpResponse::InternalServerError()
                        .body("Application saved but agent unavailable")
                }
            }
        }
        Err(error) => {
            eprintln!("DB Error: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub async fn get_programs(pool: web::Data<PgPool>) -> impl Responder {
    match get_applications(pool.get_ref()).await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(error) => {
            eprintln!("DB Error: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub async fn get_program(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let app_id = path.into_inner();
    println!("app id: {}", app_id);
    match get_application(pool.get_ref(), app_id).await {
        Ok(app) => HttpResponse::Ok().json(app),
        Err(error) => match error {
            sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
            _ => {
                eprintln!("DB Error: {}", error);
                return HttpResponse::InternalServerError().finish();
            }
        },
    }
}

pub async fn patch_program(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    edited_app_info: web::Json<PatchApplication>,
) -> impl Responder {
    let app_id = path.into_inner();
    println!("patch app id: {}", app_id);

    // If stopping, update DB to STOPPED FIRST, then kill the process
    // This prevents the agent from restarting the app after kill
    if matches!(edited_app_info.status, Some(AppStatus::STOPPED)) {
        match get_application(pool.get_ref(), app_id).await {
            Ok(app) => {
                // Update DB to STOPPED before killing so agent sees STOPPED status
                if let Err(e) = patch_application(pool.get_ref(), app_id, &edited_app_info).await {
                    eprintln!("DB Error: {}", e);
                    return HttpResponse::InternalServerError().finish();
                }

                // Now kill the process
                if let Some(pid) = app.pid {
                    let client = Client::new();
                    let agent_url = "http://127.0.0.1:8001/stop";
                    let body = serde_json::json!({ "pid": pid });
                    if let Err(e) = client.post(agent_url).json(&body).send().await {
                        eprintln!("Failed to contact agent to kill process: {}", e);
                    }
                }

                return HttpResponse::Ok().body(format!(
                    "Application Program ID = {} stopped successfully",
                    app_id
                ));
            }
            Err(e) => {
                eprintln!("Could not fetch app to get PID: {}", e);
            }
        }
    }

    match patch_application(pool.get_ref(), app_id, &edited_app_info).await {
        Ok(_) => HttpResponse::Ok().body(format!(
            "Application Program ID = {} Information Successfully Updated",
            app_id
        )),
        Err(error) => {
            eprintln!("DB Error: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub async fn delete_program(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let app_id = path.into_inner();
    match delete_application(pool.get_ref(), app_id).await {
        Ok(_) => HttpResponse::Ok().body(format!("Application {} deleted", app_id)),
        Err(e) => {
            eprintln!("DB Error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_live_status(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let app_id = path.into_inner();

    let app = match get_application(pool.get_ref(), app_id).await {
        Ok(app) => app,
        Err(_) => return HttpResponse::NotFound().body("Application not found"),
    };

    let live_status = match app.pid {
        Some(pid) => {
            let client = Client::new();
            let agent_url = format!("http://127.0.0.1:8001/status/{}", pid);
            match client.get(&agent_url).send().await {
                Ok(res) if res.status().is_success() => {
                    let body: serde_json::Value = res.json().await.unwrap_or_default();
                    body.get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_string()
                }
                _ => "UNKNOWN".to_string(),
            }
        }
        None => "STOPPED".to_string(),
    };

    // Auto-correct DB if status has diverged
    if live_status == "STOPPED" && matches!(app.status, AppStatus::RUNNING) {
        let patch = PatchApplication {
            name: None,
            command: None,
            port: None,
            working_dir: None,
            status: Some(AppStatus::STOPPED),
            pid: None,
            env_vars: None,
        };
        let _ = patch_application(pool.get_ref(), app_id, &patch).await;
    }

    HttpResponse::Ok().json(serde_json::json!({
        "id": app_id,
        "name": app.name,
        "status": live_status,
        "pid": app.pid,
        "port": app.port,
        "command": app.command,
    }))
}

pub async fn redeploy_program(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    let app_id = path.into_inner();
    println!("Redeploying app: {}", app_id);

    let mut app = match get_application(pool.get_ref(), app_id).await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("DB Error: {}", e);
            return HttpResponse::NotFound().body("Application not found");
        }
    };

    // Kill the old process if running
    if let Some(pid) = app.pid {
        kill_app(pid).await;
    }

    // Update port from paas.toml if provided
    if let Some(port) = body.get("port").and_then(|p| p.as_i64()) {
        app.port = port as i32;
    }

    // Clear PID explicitly and reset status to PENDING
    if let Err(e) = clear_pid(pool.get_ref(), app_id).await {
        eprintln!("Failed to clear PID: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    let patch = PatchApplication {
        name: None,
        command: None,
        port: None,
        working_dir: None,
        status: Some(AppStatus::PENDING),
        pid: None,
        env_vars: None,
    };
    if let Err(e) = patch_application(pool.get_ref(), app_id, &patch).await {
        eprintln!("Failed to reset app status: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // Start fresh process
    let port = app.port;
    start_app(app.clone()).await;

    HttpResponse::Ok().json(serde_json::json!({
        "port": port,
        "name": app.name,
    }))
}
