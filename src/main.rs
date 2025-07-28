use estuary::{database::DatabaseFactory, middleware, routes, state::AppState};
use std::net::SocketAddr;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .map_err(|e| format!("Invalid PORT value: {}. PORT must be a number between 0-65535", e))?;
    
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
    
    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
        return Err(format!("Server failed: {}", e).into());
    }

    Ok(())
}
