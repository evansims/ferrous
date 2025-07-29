use axum::{body::Body, http::Request};
use estuary::{
    db::{InMemoryRepository, ItemRepository, MetricsRepository},
    models::{CreateItemRequest, Item},
    state::SharedState,
};
use std::sync::Arc;

/// Create a test repository instance
pub fn create_test_repo() -> Arc<dyn ItemRepository> {
    // Wrap with metrics tracking like in production
    let base_repo = Arc::new(InMemoryRepository::new());
    Arc::new(MetricsRepository::new(base_repo))
}

/// Create a test app state
pub fn create_test_state() -> SharedState {
    let repo = create_test_repo();
    estuary::state::AppState::shared(repo)
}

/// Create a test item request
#[allow(dead_code)]
pub fn create_test_item_request(name: &str, description: Option<&str>) -> CreateItemRequest {
    CreateItemRequest {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
    }
}

/// Create and insert a test item into the repository
#[allow(dead_code)]
pub async fn create_test_item(
    repo: &Arc<dyn ItemRepository>,
    name: &str,
    description: Option<&str>,
) -> Item {
    let request = create_test_item_request(name, description);
    repo.create(request).await.unwrap()
}

/// Create multiple test items
#[allow(dead_code)]
pub async fn create_test_items(repo: &Arc<dyn ItemRepository>, count: usize) -> Vec<Item> {
    let mut items = Vec::new();
    for i in 0..count {
        let item = create_test_item(
            repo,
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
