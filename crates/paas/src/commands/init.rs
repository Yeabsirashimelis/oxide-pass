use std::{
    fs::{self},
    io::{self, Write},
    path::{Path, PathBuf},
};

fn prompt_command(runtime: &str) -> String {
    print!("Could not auto-detect entry point for {}. Enter the run command: ", runtime);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn detect_go_command() -> String {
    if Path::new("main.go").exists() {
        return "go run main.go".to_string();
    }

    if Path::new("cmd").exists() {
        let cmd_entry = std::fs::read_dir("cmd")
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.path().join("main.go").exists())
                    .map(|e| e.file_name().to_string_lossy().to_string())
            });

        if let Some(subdir) = cmd_entry {
            return format!("go run ./cmd/{}/...", subdir);
        }
    }

    prompt_command("Go")
}

pub fn init_project() -> anyhow::Result<()> {
    println!("initializing a project");

    let runtime = if Path::new("package.json").exists() {
        "node"
    } else if Path::new("Cargo.toml").exists() {
        "rust"
    } else if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
        "python"
    } else if Path::new("go.mod").exists() {
        "go"
    } else {
        "unknown"
    };

    if runtime == "unknown" {
        println!("⚠ Could not detect project type. Please create a paas.toml manually");
        return Ok(());
    }

    let current_dir: PathBuf = std::env::current_dir()?;
    let folder_name = current_dir
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Could not get folder name"))?
        .to_string_lossy()
        .to_string();

    let command = match runtime {
        "node" => "npm run dev".to_string(),
        "rust" => "cargo run".to_string(),
        "python" => "python main.py".to_string(),
        "go" => detect_go_command(),
        _ => prompt_command(runtime),
    };

    let config_content = format!(
        "name = \"{}\"\nruntime = \"{}\"\ncommand = \"{}\"\nport = 3000\n",
        folder_name, runtime, command
    );

    if Path::new("paas.toml").exists() {
        println!("⚠ paas.toml already exists, skipping creation.");
    } else {
        fs::write("paas.toml", config_content)?;
        println!("✔ Created paas.toml");
    }

    println!("✔ Paas project initialized");
    println!("Project name: {}", folder_name);
    println!("Runtime: {}", runtime);
    println!("Command: {}", command);
    println!();
    println!("Note: Make sure your app listens on the port specified in paas.toml.");
    match runtime {
        "node" => println!("  For Next.js/Vite: set port in your package.json script, e.g: \"next dev --port 3000\""),
        "go" => println!("  For Go: make sure your app listens on the port in paas.toml, e.g: http.ListenAndServe(\":3000\", nil)"),
        "python" => println!("  For Python: make sure your app listens on the port in paas.toml, e.g: app.run(port=3000)"),
        "rust" => println!("  For Rust: make sure your app listens on the port in paas.toml"),
        _ => {}
    }

    Ok(())
}
