use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use reqwest::Client;
use shared::{Application, NewAppLog};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[cfg(windows)]
mod job_object {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_BASIC_LIMIT_INFORMATION,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Threading::OpenProcess;
    use windows::Win32::System::Threading::{PROCESS_ALL_ACCESS};

    pub fn assign_process_to_job(pid: u32) {
        unsafe {
            // Create a new Job Object
            let job = CreateJobObjectW(None, None).unwrap();

            // Set KILL_ON_JOB_CLOSE so all processes in job die when agent exits
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
                BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                    LimitFlags: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                    ..Default::default()
                },
                ..Default::default()
            };

            let _ = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &mut info as *mut _ as *mut _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            // Open the process and assign to job
            let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid).unwrap();
            let _ = AssignProcessToJobObject(job, HANDLE(process.0));

            println!("Process {} assigned to Job Object (will die with agent)", pid);
        }
    }
}

const MAX_RETRIES: u32 = 3;

fn detect_port(line: &str) -> Option<i32> {
    let line_lower = line.to_lowercase();

    // Ignore error/warning lines
    if line_lower.contains("error")
        || line_lower.contains("failed")
        || line_lower.contains("refused")
        || line_lower.contains("dial")
        || line_lower.contains("connect")
    {
        return None;
    }

    // Must contain a signal that the server is ready/listening
    let is_server_ready = line_lower.contains("listening")
        || line_lower.contains("started")
        || line_lower.contains("ready")
        || line_lower.contains("running")
        || line_lower.contains("serving")
        || line_lower.contains("local:")
        || line_lower.contains("localhost")
        || line_lower.contains("127.0.0.1");

    if !is_server_ready {
        return None;
    }

    // Pattern 1: "localhost:3000" or "127.0.0.1:3000"
    for prefix in &["localhost:", "127.0.0.1:"] {
        if let Some(idx) = line.find(prefix) {
            let after = &line[idx + prefix.len()..];
            let port_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(port) = port_str.parse::<i32>() {
                if port > 1024 && port < 65536 {
                    return Some(port);
                }
            }
        }
    }

    // Pattern 2: "listening on :8080" or ":8080" (Go style)
    if line_lower.contains("listening") || line_lower.contains("serving") {
        if let Some(colon_pos) = line.rfind(':') {
            let after = &line[colon_pos + 1..];
            let port_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(port) = port_str.parse::<i32>() {
                if port > 1024 && port < 65536 {
                    return Some(port);
                }
            }
        }
    }

    // Pattern 3: "port 3000" or "PORT 3000" (Express, etc.)
    if line_lower.contains("port") {
        let words: Vec<&str> = line.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase() == "port" {
                if let Some(next) = words.get(i + 1) {
                    let port_str: String = next.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(port) = port_str.parse::<i32>() {
                        if port > 1024 && port < 65536 {
                            return Some(port);
                        }
                    }
                }
            }
        }
    }

    None
}

async fn update_status(app_id: uuid::Uuid, status: &str) {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}", app_id);
    let _ = client
        .patch(&url)
        .json(&serde_json::json!({ "status": status }))
        .send()
        .await;
}

async fn send_log(log: NewAppLog) {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/logs", log.app_id);
    if let Err(e) = client.post(&url).json(&log).send().await {
        eprintln!("Failed to send log to paasd: {}", e);
    }
}

async fn spawn_app(app: Application, attempt: u32) {
    let app_id = app.id.unwrap();
    let parts: Vec<&str> = app.command.split_whitespace().collect();
    if parts.is_empty() {
        eprintln!("Empty command");
        return;
    }

    // On Windows, npm/npx/yarn are .cmd files and need special handling
    let raw_program = parts[0].to_string();
    #[cfg(target_os = "windows")]
    let program = match raw_program.as_str() {
        "npm" => "npm.cmd".to_string(),
        "npx" => "npx.cmd".to_string(),
        "yarn" => "yarn.cmd".to_string(),
        "pnpm" => "pnpm.cmd".to_string(),
        _ => raw_program,
    };
    #[cfg(not(target_os = "windows"))]
    let program = raw_program;

    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(&app.working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(env_obj) = &app.env_vars {
        if let Some(map) = env_obj.as_object() {
            for (key, val) in map {
                if let Some(val_str) = val.as_str() {
                    cmd.env(key, val_str);
                }
            }
        }
    }

    let mut process = match cmd.spawn() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to execute process: {}", e);
            update_status(app_id, "CRASHED").await;
            return;
        }
    };

    let pid = process.id().unwrap_or(0);
    println!("Application started with PID: {} (attempt {}/{})", pid, attempt, MAX_RETRIES);

    // Assign to Job Object so child processes die when agent stops (Windows only)
    #[cfg(windows)]
    job_object::assign_process_to_job(pid);

    // Send PID and update status to RUNNING
    let client_pid = Client::new();
    let patch_url = format!("http://127.0.0.1:8080/apps/{}", app_id);
    let patch_body = serde_json::json!({ "pid": pid as i32, "status": "RUNNING" });
    if let Err(e) = client_pid.patch(&patch_url).json(&patch_body).send().await {
        eprintln!("Failed to update PID and status: {}", e);
    }

    let stdout = process.stdout.take();
    let stderr = process.stderr.take();
    let app_id_stdout = app_id;
    let app_id_stderr = app_id;

    if let Some(stdout) = stdout {
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[stdout] {}", line);
                if let Some(port) = detect_port(&line) {
                    println!("Detected app running on port {}", port);
                    let client = Client::new();
                    let url = format!("http://127.0.0.1:8080/apps/{}", app_id_stdout);
                    let _ = client
                        .patch(&url)
                        .json(&serde_json::json!({ "port": port }))
                        .send()
                        .await;
                }
                send_log(NewAppLog {
                    app_id: app_id_stdout,
                    stream: "stdout".to_string(),
                    message: line,
                })
                .await;
            }
        });
    }

    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[stderr] {}", line);
                if let Some(port) = detect_port(&line) {
                    println!("Detected app running on port {}", port);
                    let client = Client::new();
                    let url = format!("http://127.0.0.1:8080/apps/{}", app_id_stderr);
                    let _ = client
                        .patch(&url)
                        .json(&serde_json::json!({ "port": port }))
                        .send()
                        .await;
                }
                send_log(NewAppLog {
                    app_id: app_id_stderr,
                    stream: "stderr".to_string(),
                    message: line,
                })
                .await;
            }
        });
    }

    let status = process.wait().await;

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("Application exited cleanly.");
            update_status(app_id, "STOPPED").await;
        }
        _ => {
            if attempt < MAX_RETRIES {
                println!(
                    "Application crashed! Restarting... (attempt {}/{})",
                    attempt + 1,
                    MAX_RETRIES
                );
                send_log(NewAppLog {
                    app_id,
                    stream: "stderr".to_string(),
                    message: format!(
                        "[PaaS] App crashed. Restarting... (attempt {}/{})",
                        attempt + 1,
                        MAX_RETRIES
                    ),
                })
                .await;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                Box::pin(spawn_app(app, attempt + 1)).await;
            } else {
                eprintln!(
                    "Application crashed after {} attempts. Marking as CRASHED.",
                    MAX_RETRIES
                );
                send_log(NewAppLog {
                    app_id,
                    stream: "stderr".to_string(),
                    message: format!(
                        "[PaaS] App crashed after {} attempts. Giving up.",
                        MAX_RETRIES
                    ),
                })
                .await;
                update_status(app_id, "CRASHED").await;
            }
        }
    }
}

async fn run_program(app: web::Json<Application>) -> impl Responder {
    let app = app.into_inner();
    println!("Starting application: {}", app.name);
    println!("Working directory: {}", app.working_dir);
    println!("Command: {}", app.command);

    tokio::spawn(async move {
        spawn_app(app, 1).await;
    });

    HttpResponse::Ok().finish()
}

async fn stop_program(body: web::Json<serde_json::Value>) -> impl Responder {
    let pid = match body.get("pid").and_then(|p| p.as_i64()) {
        Some(p) => p as u32,
        None => return HttpResponse::BadRequest().body("Missing pid"),
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
        Ok(_) => {
            println!("Process {} killed successfully", pid);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            eprintln!("Failed to kill process {}: {}", pid, e);
            HttpResponse::InternalServerError().finish()
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
