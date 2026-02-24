use std::process::Command;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use shared::Application;

async fn run_program(app: web::Json<Application>) -> impl Responder {
    println!("Starting application: {}", app.name);
    println!("Working directory: {}", app.working_dir);
    println!("Command: {}", app.command);

    // Parse the command string into program and arguments
    let parts: Vec<&str> = app.command.split_whitespace().collect();
    
    if parts.is_empty() {
        eprintln!("Empty command provided");
        return HttpResponse::BadRequest().body("Empty command");
    }

    let program = parts[0];
    let args = &parts[1..];

    // Adjust for Windows if needed (e.g., npm -> npm.cmd)
    #[cfg(target_os = "windows")]
    let program = if program == "npm" {
        "npm.cmd"
    } else {
        program
    };

    let child = Command::new(program)
        .args(args)
        .current_dir(&app.working_dir)
        .spawn();

    match child {
        Ok(process) => {
            println!("Application started with PID: {:?}", process.id());
            HttpResponse::Ok().body(format!("Started with PID: {}", process.id()))
        }
        Err(e) => {
            eprintln!("Failed to execute process: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to start: {}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = ("127.0.0.1", 8001);
    println!("app is bound to http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || App::new().route("/run", web::post().to(run_program)))
        .bind(addr)?
        .run()
        .await
}
