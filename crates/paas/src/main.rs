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

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init => println!("this is init command"),
        Commands::Deploy => println!("this is deploy command"),
        Commands::Status => println!("this is status command"),
        Commands::Logs => println!("this is logs command"),
        Commands::Stop => println!("this is stop command"),
    };
}
