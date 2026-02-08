use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Deploy,
    Status,
    Logs,
    Stop,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
