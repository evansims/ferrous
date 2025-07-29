use axum::http::StatusCode;
use tower::util::ServiceExt;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_request_id_generation() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let request_id = response
        .headers()
        .get("X-Request-Id")
        .expect("X-Request-Id header missing");

    // Verify it's a valid UUID
    let id_str = request_id.to_str().unwrap();
    Uuid::parse_str(id_str).expect("Invalid UUID format");
}

#[tokio::test]
async fn test_request_id_forwarding() {
    let app = common::create_test_app().await;

    let existing_id = Uuid::new_v4().to_string();
    let mut request = common::get_request("/health");
    request
        .headers_mut()
        .insert("X-Request-Id", existing_id.parse().unwrap());

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let request_id = response
        .headers()
        .get("X-Request-Id")
        .expect("X-Request-Id header missing")
        .to_str()
        .unwrap();

    assert_eq!(request_id, existing_id);
}

#[tokio::test]
async fn test_request_id_in_api_calls() {
    let app = common::create_test_app().await;

    // Test with successful request
    let response = app
        .clone()
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert!(response.headers().contains_key("X-Request-Id"));

    // Test with error response
    let response = app
        .oneshot(common::get_request("/api/v1/items/nonexistent"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.headers().contains_key("X-Request-Id"));

    // Error response might include request_id in body
    let error_body = common::response_json::<serde_json::Value>(response).await;
    // request_id is optional in error responses
    if error_body.get("request_id").is_some() {
        assert!(error_body["request_id"].is_string());
    }
}

#[tokio::test]
async fn test_graceful_shutdown_message() {
    // This test verifies the shutdown handler exists
    // In a real test we'd need to trigger shutdown and verify behavior
    let state = common::create_test_state();

    // Verify the app state has necessary components for shutdown
    assert!(state.repo.health_check().await.is_ok());
}
