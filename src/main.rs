use estuary::{
    database::DatabaseFactory, handlers::health::APP_START_TIME, metrics, middleware, routes,
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

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "estuary=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let db = match DatabaseFactory::create().await {
        Ok(db) => {
            info!("Database initialized successfully");
            db
        }
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(format!("Database initialization failed: {}", e).into());
        }
    };

    // Create shared application state
    let state = AppState::shared(db);

    // Build application with routes and middleware
    let app = middleware::add_middleware(routes::create_routes(state));

    // Configure socket address
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .map_err(|e| {
            format!(
                "Invalid PORT value: {}. PORT must be a number between 0-65535",
                e
            )
        })?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
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

    // Configure shutdown timeout
    let _shutdown_timeout = std::env::var("SHUTDOWN_TIMEOUT_SECONDS")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()
        .unwrap_or(30);

    // Create the server
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

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
async fn shutdown_signal() {
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

    warn!("Shutdown signal received, waiting for existing connections to close...");
}
