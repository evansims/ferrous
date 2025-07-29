use crate::common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("healthy"));
}

#[tokio::test]
async fn test_create_item() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "name": "Test Item",
        "description": "Test Description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let item: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(item["name"], "Test Item");
    assert_eq!(item["description"], "Test Description");
    assert!(item["id"].is_string());
    assert!(item["created_at"].is_string());
    assert!(item["updated_at"].is_string());
}

#[tokio::test]
async fn test_create_item_without_description() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "name": "Test Item"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let item: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(item["name"], "Test Item");
    assert!(item["description"].is_null());
}

#[tokio::test]
async fn test_get_item() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create an item first
    let created = common::create_test_item(&state.db, "Test Item", Some("Test Description")).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/items/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let item: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(item["id"], created.id);
    assert_eq!(item["name"], "Test Item");
    assert_eq!(item["description"], "Test Description");
}

#[tokio::test]
async fn test_get_nonexistent_item() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/items/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_item() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create an item first
    let created =
        common::create_test_item(&state.db, "Original Name", Some("Original Description")).await;

    let update_body = json!({
        "name": "Updated Name"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/v1/items/{}", created.id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let item: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(item["name"], "Updated Name");
    assert_eq!(item["description"], "Original Description");
}

#[tokio::test]
async fn test_delete_item() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create an item first
    let created = common::create_test_item(&state.db, "To Delete", None).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/items/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify item is deleted
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/items/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_items() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create multiple items
    common::create_test_items(&state.db, 5).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(list_response["items"].as_array().unwrap().len(), 5);
    assert_eq!(list_response["total"], 5);
    assert_eq!(list_response["limit"], 20);
    assert_eq!(list_response["offset"], 0);
}

#[tokio::test]
async fn test_list_items_with_pagination() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create multiple items
    common::create_test_items(&state.db, 5).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/items?limit=2&offset=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(list_response["items"].as_array().unwrap().len(), 2);
    assert_eq!(list_response["total"], 5);
    assert_eq!(list_response["limit"], 2);
    assert_eq!(list_response["offset"], 2);
}
