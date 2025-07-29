use axum::http::StatusCode;
use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn test_health_check() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_body_string(response).await;
    assert!(body.contains("healthy"));
}

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
    // The readiness endpoint returns a simpler structure
    assert_eq!(body["status"], "ready");
    if body.get("components").is_some() {
        assert_eq!(body["components"]["database"]["status"], "healthy");
    }
    assert!(body["timestamp"].is_string());
}

#[tokio::test]
async fn test_comprehensive_health_endpoint() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_json::<serde_json::Value>(response).await;

    // Check top-level structure
    assert_eq!(body["status"], "healthy");
    assert!(body["timestamp"].is_string());
    assert!(body["version"].is_string());
    assert!(body["uptime_seconds"].is_number());

    // Check components
    if body.get("components").is_some() {
        assert_eq!(body["components"]["database"]["status"], "healthy");
    }

    // Check system info
    if body.get("system").is_some() {
        assert!(body["system"]["memory_used_mb"].is_number());
        assert!(body["system"]["cpu_count"].is_u64());
    }
}
