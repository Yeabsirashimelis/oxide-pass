use crate::{
    cli::{Commands, parse_cli},
    commands::{
        deploy::deploy_project, init::init_project, logs::show_logs,
        redeploy::redeploy_project, status::check_status, stop::stop_application,
    },
};

mod cli;
mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_cli();

    match args.command {
        Commands::Init => init_project(),
        Commands::Deploy => deploy_project().await,
        Commands::Redeploy => redeploy_project().await,
        Commands::Status => check_status().await,
        Commands::Logs { follow } => show_logs(follow).await,
        Commands::Stop => stop_application().await,
    }
}
