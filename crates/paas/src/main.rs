use crate::{
    cli::{Commands, parse_cli},
    commands::init::init_project,
};

mod cli;
mod commands;

fn main() -> anyhow::Result<()> {
    let args = parse_cli();

    match args.command {
        Commands::Init => init_project(),
        Commands::Deploy => println!("this is deploy command"),
        Commands::Status => println!("this is status command"),
        Commands::Logs => println!("this is logs command"),
        Commands::Stop => println!("this is stop command"),
    };

    Ok(())
}
