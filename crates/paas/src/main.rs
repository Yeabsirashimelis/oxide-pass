use anyhow::Ok;

use crate::{
    cli::{Commands, parse_cli},
    commands::{deploy::deploy_project, init::init_project, status::check_status},
};

mod cli;
mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_cli();

    match args.command {
        Commands::Init => init_project(),
        Commands::Deploy => deploy_project().await,
        Commands::Status => check_status().await,
        Commands::Logs => Ok(()),
        Commands::Stop => Ok(()),
    }
}
