use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use reqwest::Client;
use shared::{Application, NewAppLog};

async fn run_program(app: web::Json<Application>) -> impl Responder {
    println!("Starting application: {}", app.name);
    println!("Working directory: {}", app.working_dir);
    println!("Command: {}", app.command);

    let parts: Vec<&str> = app.command.split_whitespace().collect();

    if parts.is_empty() {
        eprintln!("Empty command provided");
        return HttpResponse::BadRequest().body("Empty command");
    }

    let program = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    let working_dir = app.working_dir.clone();
    let app_id = app.id.unwrap();

    #[cfg(target_os = "windows")]
    let program = if program == "npm" {
        "npm.cmd".to_string()
    } else {
        program
    };

    let child = Command::new(&program)
        .args(&args)
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut process) => {
            let pid = process.id();
            println!("Application started with PID: {:?}", pid);

            // Send PID and update status to RUNNING in paasd
            let client_pid = Client::new();
            let patch_url = format!("http://127.0.0.1:8080/apps/{}", app_id);
            let patch_body = serde_json::json!({ "pid": pid as i32, "status": "RUNNING" });
            if let Err(e) = client_pid.patch(&patch_url).json(&patch_body).send().await {
                eprintln!("Failed to update PID and status in paasd: {}", e);
            }

            let stdout = process.stdout.take();
            let stderr = process.stderr.take();

            // Spawn a task to read stdout and send to paasd
            if let Some(stdout) = stdout {
                let app_id_clone = app_id;
                tokio::task::spawn_blocking(move || {
                    let reader = BufReader::new(stdout);
                    let rt = tokio::runtime::Handle::current();
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                println!("[stdout] {}", line);

                                // Detect the actual port the app is running on
                                // Matches lines like: "- Local:        http://localhost:3000"
                                if line.contains("Local:") && line.contains("localhost:") {
                                    if let Some(port_str) = line
                                        .split("localhost:")
                                        .nth(1)
                                        .and_then(|s| s.split('/').next())
                                        .and_then(|s| s.trim().parse::<i32>().ok().map(|p| p.to_string()))
                                    {
                                        let client = Client::new();
                                        let patch_url = format!("http://127.0.0.1:8080/apps/{}", app_id_clone);
                                        let patch_body = serde_json::json!({ "port": port_str.parse::<i32>().unwrap_or(3000) });
                                        rt.block_on(async {
                                            if let Err(e) = client.patch(&patch_url).json(&patch_body).send().await {
                                                eprintln!("Failed to update actual port: {}", e);
                                            } else {
                                                println!("Detected app running on port {}", port_str);
                                            }
                                        });
                                    }
                                }

                                let log = NewAppLog {
                                    app_id: app_id_clone,
                                    stream: "stdout".to_string(),
                                    message: line,
                                };
                                rt.block_on(send_log(log));
                            }
                            Err(e) => eprintln!("Error reading stdout: {}", e),
                        }
                    }
                });
            }

            // Spawn a task to read stderr and send to paasd
            if let Some(stderr) = stderr {
                let app_id_clone = app_id;
                tokio::task::spawn_blocking(move || {
                    let reader = BufReader::new(stderr);
                    let rt = tokio::runtime::Handle::current();
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                eprintln!("[stderr] {}", line);
                                let log = NewAppLog {
                                    app_id: app_id_clone,
                                    stream: "stderr".to_string(),
                                    message: line,
                                };
                                rt.block_on(send_log(log));
                            }
                            Err(e) => eprintln!("Error reading stderr: {}", e),
                        }
                    }
                });
            }

            HttpResponse::Ok().body(format!("Started with PID: {}", pid))
        }
        Err(e) => {
            eprintln!("Failed to execute process: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to start: {}", e))
        }
    }
}

async fn stop_program(body: web::Json<serde_json::Value>) -> impl Responder {
    let pid = match body.get("pid").and_then(|p| p.as_i64()) {
        Some(pid) => pid as u32,
        None => {
            eprintln!("No PID provided");
            return HttpResponse::BadRequest().body("No PID provided");
        }
    };

    println!("Killing process with PID: {}", pid);

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F", "/T"])
        .output();

    #[cfg(not(target_os = "windows"))]
    let result = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("Process {} killed successfully", pid);
            HttpResponse::Ok().body(format!("Process {} stopped", pid))
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to kill process {}: {}", pid, err);
            HttpResponse::InternalServerError().body(format!("Failed to kill process: {}", err))
        }
        Err(e) => {
            eprintln!("Error killing process {}: {}", pid, e);
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }
}

async fn check_status(path: web::Path<u32>) -> impl Responder {
    let pid = path.into_inner();

    #[cfg(target_os = "windows")]
    let is_alive = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false);

    #[cfg(not(target_os = "windows"))]
    let is_alive = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if is_alive {
        HttpResponse::Ok().json(serde_json::json!({ "status": "RUNNING" }))
    } else {
        HttpResponse::Ok().json(serde_json::json!({ "status": "STOPPED" }))
    }
}

async fn send_log(log: NewAppLog) {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/logs", log.app_id);
    if let Err(e) = client.post(&url).json(&log).send().await {
        eprintln!("Failed to send log to paasd: {}", e);
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = ("127.0.0.1", 8001);
    println!("app is bound to http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .route("/run", web::post().to(run_program))
            .route("/stop", web::post().to(stop_program))
            .route("/status/{pid}", web::get().to(check_status))
    })
    .bind(addr)?
    .run()
    .await
}
