use axum::{body::Body, http::Request};
use estuary::{
    database::{implementations::in_memory::InMemoryDatabase, Database, MetricsDatabase},
    models::{CreateItemRequest, Item},
    state::SharedState,
};
use std::sync::Arc;

/// Create a test database instance
pub fn create_test_db() -> Arc<dyn Database> {
    // Wrap with metrics tracking like in production
    let base_db = Arc::new(InMemoryDatabase::new());
    Arc::new(MetricsDatabase::new(base_db))
}

/// Create a test app state
pub fn create_test_state() -> SharedState {
    let db = create_test_db();
    estuary::state::AppState::shared(db)
}

/// Create a test item request
#[allow(dead_code)]
pub fn create_test_item_request(name: &str, description: Option<&str>) -> CreateItemRequest {
    CreateItemRequest {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
    }
}

/// Create and insert a test item into the database
#[allow(dead_code)]
pub async fn create_test_item(
    db: &Arc<dyn Database>,
    name: &str,
    description: Option<&str>,
) -> Item {
    let request = create_test_item_request(name, description);
    db.items().create(request).await.unwrap()
}

/// Create multiple test items
#[allow(dead_code)]
pub async fn create_test_items(db: &Arc<dyn Database>, count: usize) -> Vec<Item> {
    let mut items = Vec::new();
    for i in 0..count {
        let item = create_test_item(
            db,
            &format!("Test Item {}", i),
            Some(&format!("Description for item {}", i)),
        )
        .await;
        items.push(item);
    }
    items
}

/// Create a test app for integration testing
pub async fn create_test_app() -> axum::Router {
    // Initialize metrics for tests
    estuary::metrics::init_metrics();

    let state = create_test_state();
    let app = estuary::routes::create_routes(state);
    estuary::middleware::add_middleware(app)
}

/// Create a GET request
#[allow(dead_code)]
pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

/// Create a POST request with JSON body
#[allow(dead_code)]
pub fn post_request(uri: &str, json: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

/// Parse response body as JSON
#[allow(dead_code)]
pub async fn response_json<T>(response: axum::response::Response) -> T
where
    T: serde::de::DeserializeOwned,
{
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Create a PUT request with JSON body
#[allow(dead_code)]
pub fn put_request(uri: &str, json: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

/// Create a DELETE request
#[allow(dead_code)]
pub fn delete_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

/// Get response body as string
#[allow(dead_code)]
pub async fn response_body_string(response: axum::response::Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}
