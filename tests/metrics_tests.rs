use axum::http::StatusCode;
use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn test_metrics_endpoint_exists() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/metrics")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/plain"));
}

#[tokio::test]
async fn test_metrics_content() {
    let app = common::create_test_app().await;

    // Make a few requests to generate metrics
    let _ = app
        .clone()
        .oneshot(common::get_request("/health"))
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    // Get metrics
    let response = app.oneshot(common::get_request("/metrics")).await.unwrap();

    let body = common::response_body_string(response).await;

    // Check for expected metric families that should always be present
    assert!(body.contains("# TYPE http_request_duration_seconds histogram"));
    assert!(body.contains("# TYPE http_requests_total counter"));
    assert!(body.contains("# TYPE database_query_duration_seconds histogram"));
    assert!(body.contains("# TYPE database_queries_total counter"));
    assert!(body.contains("# TYPE database_connections_active gauge"));

    // Business metrics will only appear after they've been incremented
    // We'll test those separately in test_metrics_tracking_business_operations
}

#[tokio::test]
async fn test_metrics_tracking_http_requests() {
    let app = common::create_test_app().await;

    // Make some requests
    for _ in 0..3 {
        let _ = app
            .clone()
            .oneshot(common::get_request("/health"))
            .await
            .unwrap();
    }

    // Get metrics
    let response = app.oneshot(common::get_request("/metrics")).await.unwrap();

    let body = common::response_body_string(response).await;

    // Check that HTTP metrics were recorded
    assert!(body.contains("http_requests_total"));
    assert!(body.contains(r#"method="GET""#));
    assert!(body.contains(r#"endpoint="/health""#));
    assert!(body.contains(r#"status="200""#));
}

#[tokio::test]
async fn test_metrics_tracking_business_operations() {
    let app = common::create_test_app().await;

    // Create an item
    let create_request = common::post_request(
        "/api/v1/items",
        serde_json::json!({
            "name": "Metrics Test Item",
            "description": "Testing metrics"
        }),
    );
    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let item: serde_json::Value = common::response_json(create_response).await;
    let item_id = item["id"].as_str().unwrap();

    // Update the item
    let update_request = common::put_request(
        &format!("/api/v1/items/{}", item_id),
        serde_json::json!({
            "name": "Updated Metrics Test Item"
        }),
    );
    let update_response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);

    // Delete the item
    let delete_response = app
        .clone()
        .oneshot(common::delete_request(&format!(
            "/api/v1/items/{}",
            item_id
        )))
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Get metrics
    let response = app.oneshot(common::get_request("/metrics")).await.unwrap();

    let body = common::response_body_string(response).await;

    // Check business metrics - they should now appear since we've incremented them
    assert!(body.contains("items_created_total 1"));
    assert!(body.contains("items_updated_total 1"));
    assert!(body.contains("items_deleted_total 1"));
}

#[tokio::test]
async fn test_metrics_database_operations() {
    let app = common::create_test_app().await;

    // Make database queries
    let _ = app
        .clone()
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    // Get metrics
    let response = app.oneshot(common::get_request("/metrics")).await.unwrap();

    let body = common::response_body_string(response).await;

    // Check database metrics
    assert!(body.contains("database_queries_total"));
    assert!(body.contains(r#"operation="list""#));
    assert!(body.contains(r#"repository="items""#));
    assert!(body.contains(r#"status="success""#));
}
