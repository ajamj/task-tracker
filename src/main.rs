//! tt - Git-friendly personal task tracking CLI

use clap::Parser;
use tt::cli::{Cli, Commands, execute};

#[tokio::main]
async fn main() {
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
        Some(Commands::Dashboard { port }) => {
            if let Err(e) = cmd_dashboard(port).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(command) => {
            if let Err(e) = execute(command) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Run the web dashboard.
async fn cmd_dashboard(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use tt::api::{AppState, create_router};
    use std::net::SocketAddr;
    use tower_http::cors::{CorsLayer, Any};
    use http::{Method, header::HeaderValue};
    use std::env;

    let cwd = env::current_dir()?;
    let workspace = tt::storage::Workspace::load(cwd.clone())?;
    
    let state = AppState::new(cwd, workspace);

    // CORS for localhost
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = create_router(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    println!("\n🚀 tt Dashboard");
    println!("═══════════════════════════════════════");
    println!("  Running at: http://{}", addr);
    println!("  API:        http://{}/api/tasks", addr);
    println!("  Stats:      http://{}/api/stats", addr);
    println!("═══════════════════════════════════════");
    println!("\nPress Ctrl+C to stop\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
