pub mod error;
pub mod metrics;

#[cfg(test)]
mod tests;

use crate::{
    auth::{AuthConfig, JwtValidator},
    rate_limit::{RateLimitConfig, RateLimiter},
    request_id::request_id_middleware,
    version::version_middleware,
};
use axum::{middleware, Router};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

pub fn add_middleware(app: Router) -> Router {
    let rate_limiter = RateLimiter::new(RateLimitConfig::from_env());
    let jwt_validator = JwtValidator::new(AuthConfig::from_env());

    app.layer(
        ServiceBuilder::new()
            // Request ID tracking (outermost to generate ID first)
            .layer(middleware::from_fn(request_id_middleware))
            // Version extraction and validation
            .layer(middleware::from_fn(version_middleware))
            // Metrics collection
            .layer(middleware::from_fn(metrics::metrics_middleware))
            // Apply rate limiting
            .layer(middleware::from_fn(move |req, next| {
                crate::rate_limit::rate_limit_middleware(req, next, rate_limiter.clone())
            }))
            // Security headers
            .layer(middleware::from_fn(
                crate::security::security_headers_middleware,
            ))
            // JWT authentication
            .layer(middleware::from_fn(move |req, next| {
                crate::auth::auth_middleware(req, next, jwt_validator.clone())
            }))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .layer(CorsLayer::permissive()),
    )
}
