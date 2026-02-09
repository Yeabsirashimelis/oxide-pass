use anyhow::Ok;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

pub fn init_project() -> anyhow::Result<()> {
    println!("initializing a project");

    // check the type of the project (rust, node, python)

    let runtime = if Path::new("package.json").exists() {
        "node"
    } else if Path::new("Cargo.toml").exists() {
        "rust"
    } else if Path::new("pyproject.toml").exists() {
        "python"
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

    let config_content = format!(
        r#"name = "{}"
runtime = "{}"
command = "{}"
port = 3000
"#,
        folder_name,
        runtime,
        match runtime {
            "node" => "npm run dev",
            "rust" => "cargo run",
            "python" => "python main.py",
            _ => "",
        }
    );

    // write pass.toml
    if Path::new("paas.toml").exists() {
        println!("⚠ paas.toml already exists, skipping creation.");
    } else {
        fs::write("paas.toml", config_content)?;
        println!("✔ Created paas.toml");
    }

    println!("✔ Paas project initialized");
    println!("Project name: {}", folder_name);
    println!("Runtime: {}", runtime);

    Ok(())
}
