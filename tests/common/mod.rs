use estuary::{
    database::{implementations::in_memory::InMemoryDatabase, Database},
    models::{CreateItemRequest, Item},
    state::SharedState,
};
use std::sync::Arc;

/// Create a test database instance
pub fn create_test_db() -> Arc<dyn Database> {
    Arc::new(InMemoryDatabase::new())
}

/// Create a test app state
pub fn create_test_state() -> SharedState {
    let db = create_test_db();
    estuary::state::AppState::shared(db)
}

/// Create a test item request
pub fn create_test_item_request(name: &str, description: Option<&str>) -> CreateItemRequest {
    CreateItemRequest {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
    }
}

/// Create and insert a test item into the database
pub async fn create_test_item(db: &Arc<dyn Database>, name: &str, description: Option<&str>) -> Item {
    let request = create_test_item_request(name, description);
    db.items().create(request).await.unwrap()
}

/// Create multiple test items
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
    let state = create_test_state();
    estuary::routes::create_routes(state)
}