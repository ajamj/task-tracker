//! tt - Git-friendly personal task tracking CLI

use clap::Parser;
use tt::cli::{Cli, Commands, execute};

fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            println!("tt - Git-friendly personal task tracking CLI");
            println!("\nRun 'tt --help' to see available commands.");
        }
        Some(command) => {
            if let Err(e) = execute(command) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
