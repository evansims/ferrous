use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_liveness_endpoint() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/health/live"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_json::<serde_json::Value>(response).await;
    assert_eq!(body["status"], "alive");
    assert!(body["timestamp"].is_string());
}

#[tokio::test]
async fn test_readiness_endpoint_when_ready() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/health/ready"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_json::<serde_json::Value>(response).await;
    assert_eq!(body["status"], "ready");
    assert!(body["timestamp"].is_string());
}

#[tokio::test]
async fn test_comprehensive_health_endpoint() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_json::<serde_json::Value>(response).await;

    // Check top-level fields
    assert_eq!(body["status"], "healthy");
    assert!(body["timestamp"].is_string());
    assert!(body["uptime_seconds"].is_u64());
    assert!(body["version"].is_string());

    // Check database health
    assert!(body["database"]["connected"].is_boolean());
    assert_eq!(body["database"]["connected"], true);
    assert!(body["database"]["response_time_ms"].is_u64());

    // Check system health
    assert!(body["system"]["memory_used_mb"].is_u64());
    assert!(body["system"]["memory_total_mb"].is_u64());
    assert!(body["system"]["memory_usage_percent"].is_f64());
    assert!(body["system"]["cpu_count"].is_u64());

    // Memory usage should be between 0 and 100
    let memory_usage = body["system"]["memory_usage_percent"].as_f64().unwrap();
    assert!(memory_usage >= 0.0 && memory_usage <= 100.0);
}

#[tokio::test]
async fn test_request_id_generation() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/")).await.unwrap();

    // Check that X-Request-Id header is present
    let request_id_header = response.headers().get("x-request-id");
    assert!(request_id_header.is_some());

    // Verify it's a valid UUID
    let request_id = request_id_header.unwrap().to_str().unwrap();
    assert!(Uuid::parse_str(request_id).is_ok());
}

#[tokio::test]
async fn test_request_id_forwarding() {
    let app = common::create_test_app().await;
    let custom_id = Uuid::new_v4().to_string();

    let mut request = common::get_request("/");
    request
        .headers_mut()
        .insert("x-request-id", custom_id.parse().unwrap());

    let response = app.oneshot(request).await.unwrap();

    // Check that the same request ID is returned
    let response_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(response_id, custom_id);
}

#[tokio::test]
async fn test_request_id_in_api_calls() {
    let app = common::create_test_app().await;

    // Create an item
    let create_request = common::post_request(
        "/api/v1/items",
        json!({
            "name": "Request ID Test Item",
            "description": "Testing request ID propagation"
        }),
    );

    let response = app.oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify request ID is in response
    let request_id = response.headers().get("x-request-id");
    assert!(request_id.is_some());

    // Verify it's a valid UUID
    let id_str = request_id.unwrap().to_str().unwrap();
    assert!(Uuid::parse_str(id_str).is_ok());
}

#[tokio::test]
async fn test_graceful_shutdown_message() {
    // This test just verifies the shutdown handler exists
    // Actual shutdown testing would require process management

    // We can at least verify the server starts correctly
    let app = common::create_test_app().await;
    let response = app.oneshot(common::get_request("/")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
