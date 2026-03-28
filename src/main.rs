//! tt - Git-friendly personal task tracking CLI

use clap::Parser;
use tt::cli::{Cli, Commands, execute};
use tt::error::TtError;

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
                // Display error with suggestions if available
                if let Some(tt_err) = downcast_tt_error(&e) {
                    eprintln!("{}", tt_err.display_with_suggestions());
                } else {
                    eprintln!("Error: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
}

/// Try to downcast an error to TtError
fn downcast_tt_error(e: &dyn std::error::Error) -> Option<&TtError> {
    let mut source = e.source();
    while let Some(err) = source {
        if let Some(tt_err) = err.downcast_ref::<TtError>() {
            return Some(tt_err);
        }
        source = err.source();
    }
    e.downcast_ref::<TtError>()
}
