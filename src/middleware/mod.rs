pub mod auth;
pub mod error;
pub mod observability;
pub mod rate_limit;
pub mod security;
pub mod version;

#[cfg(test)]
mod tests;

use axum::{middleware, Router};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Add all middleware layers to the application
///
/// The middleware is organized into three main layers:
/// 1. Security - CORS, security headers, CSP
/// 2. Observability - Request ID, tracing, metrics
/// 3. API features - Rate limiting, authentication, versioning
pub fn add_middleware(app: Router) -> Router {
    // Load configurations
    let auth_config = auth::AuthConfig::from_env();
    let rate_limit_config = rate_limit::RateLimitConfig::from_env();
    let rate_limiter = rate_limit::RateLimiter::new(rate_limit_config);

    app.layer(
        ServiceBuilder::new()
            // Layer 1: Security (outermost)
            .layer(CorsLayer::permissive())
            .layer(middleware::from_fn(security::security_headers))
            // Layer 2: Observability
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .layer(middleware::from_fn(observability::request_id_middleware))
            .layer(middleware::from_fn(observability::metrics_middleware))
            // Layer 3: API features
            .layer(middleware::from_fn(version::version_middleware))
            .layer(middleware::from_fn(move |req, next| {
                let limiter = rate_limiter.clone();
                rate_limit::rate_limit_middleware(req, next, limiter)
            }))
            .layer(middleware::from_fn(move |req, next| {
                let config = auth_config.clone();
                auth::auth_middleware(req, next, config)
            })),
    )
}
