use ferrous::{
    config::Config, db::create_repository, handlers::APP_START_TIME, metrics, middleware, routes,
    state::AppState,
};
use std::{net::SocketAddr, time::Instant};
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize application start time for uptime tracking
    APP_START_TIME.set(Instant::now()).ok();

    // Initialize metrics
    metrics::init_metrics();

    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Load and validate configuration
    let config = match Config::load() {
        Ok(cfg) => {
            info!("Configuration loaded and validated successfully");
            cfg
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // Validate runtime dependencies
    if let Err(e) = config.validate_runtime_dependencies() {
        error!("Configuration runtime validation failed: {}", e);
        return Err(e.into());
    }

    // Removed secrets validation - use external tools for secrets management

    // Initialize tracing with configuration
    tracing_subscriber::registry()
        .with(
            config
                .logging
                .rust_log
                .parse::<tracing_subscriber::EnvFilter>()
                .unwrap_or_else(|_| "ferrous=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize repository
    let repo = create_repository(&config);
    info!("Repository initialized successfully");

    // Create shared application state
    let state = AppState::shared(repo);

    // Build application with routes and middleware
    let app = middleware::add_middleware(routes::create_routes(state));

    // Configure socket address from validated config
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Starting server on http://{}", addr);

    // Start server
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to address {}: {}", addr, e);
            return Err(format!("Cannot bind to address {}: {}", addr, e).into());
        }
    };

    info!("Server is ready to accept connections");

    // Create the server with configured shutdown
    let shutdown_config = config.shutdown.clone();
    let server =
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(shutdown_config));

    // Run the server
    info!("Server running. Press Ctrl+C to initiate graceful shutdown");

    if let Err(e) = server.await {
        error!("Server error: {}", e);
        return Err(format!("Server failed: {}", e).into());
    }

    info!("Server has shut down successfully");
    Ok(())
}

/// Handle shutdown signals
async fn shutdown_signal(shutdown_config: ferrous::config::ShutdownConfig) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received terminate signal, initiating graceful shutdown");
        },
    }

    warn!(
        "Shutdown signal received, waiting up to {} seconds for existing connections to close...",
        shutdown_config.timeout_seconds
    );
}
