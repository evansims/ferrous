use crate::{
    error::{ErrorCode, ErrorDetails, ErrorResponse, ValidationError},
    handlers::{DatabaseHealth, HealthResponse, HealthStatus, ListResponse, SystemHealth},
    models::{CreateItemRequest, Item, UpdateItemRequest},
};
use axum::{response::IntoResponse, routing::get, Json, Router};
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Ferrous API",
        version = env!("CARGO_PKG_VERSION"),
        description = "A minimal REST API service built with Rust and Axum. This documentation is for API version v1.",
        contact(
            name = "Ferrous Team",
            email = "support@example.com",
        ),
        license(
            name = "MIT",
        ),
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.example.com", description = "Production server"),
    ),
    paths(
        crate::handlers::health_check,
        crate::handlers::liveness,
        crate::handlers::readiness,
        crate::handlers::list_items,
        crate::handlers::get_item,
        crate::handlers::create_item,
        crate::handlers::update_item,
        crate::handlers::delete_item,
    ),
    components(
        schemas(
            // Models
            Item,
            CreateItemRequest,
            UpdateItemRequest,
            ListResponse,

            // Health
            HealthResponse,
            HealthStatus,
            DatabaseHealth,
            SystemHealth,

            // Errors
            ErrorResponse,
            ErrorCode,
            ErrorDetails,
            ValidationError,
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "items", description = "Item management endpoints"),
    ),
)]
pub struct ApiDoc;

/// Security addon for JWT authentication
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

/// Create documentation routes
pub fn create_docs_routes() -> Router {
    Router::new().route("/openapi.json", get(openapi_json_handler))
}

/// Serve the OpenAPI JSON spec
async fn openapi_json_handler() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}
