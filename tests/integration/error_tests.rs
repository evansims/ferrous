use crate::common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

// TODO: Enable this test when input validation is implemented in Phase 2
#[ignore]
#[tokio::test]
async fn test_create_item_missing_name() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "description": "Missing name"
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_item_invalid_json() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_nonexistent_item() {
    let app = common::create_test_app().await;

    let update_body = json!({
        "name": "Updated Name"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/items/nonexistent-id")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_item() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/items/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// TODO: Enable this test when input validation is implemented in Phase 2
#[ignore]
#[tokio::test]
async fn test_invalid_pagination_params() {
    let app = common::create_test_app().await;

    // Test negative limit
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/items?limit=-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test limit > 100
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/items?limit=101")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test invalid number format
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/items?limit=abc")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}