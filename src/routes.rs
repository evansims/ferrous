use crate::{handlers::*, openapi, state::SharedState};
use axum::{routing::get, Router};

pub fn create_routes(state: SharedState) -> Router {
    // Create stateful routes
    let api_routes = Router::new()
        // Health endpoints
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        // Metrics endpoint
        .route("/metrics", get(metrics_handler))
        // API endpoints
        .route("/api/v1/items", get(list_items).post(create_item))
        .route(
            "/api/v1/items/{id}",
            get(get_item).put(update_item).delete(delete_item),
        )
        .with_state(state);

    // Merge documentation routes (they don't need state)
    Router::new()
        .merge(openapi::create_docs_routes())
        .merge(api_routes)
}
