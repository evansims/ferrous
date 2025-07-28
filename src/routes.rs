use crate::{handlers::*, state::SharedState};
use axum::{routing::get, Router};

pub fn create_routes(state: SharedState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/api/v1/items", get(list_items).post(create_item))
        .route(
            "/api/v1/items/{id}",
            get(get_item).put(update_item).delete(delete_item),
        )
        .with_state(state)
}
