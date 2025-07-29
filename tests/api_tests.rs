use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::util::ServiceExt;

mod common;

// CREATE tests
#[tokio::test]
async fn test_create_item() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "name": "Test Item",
        "description": "Test Description"
    });

    let response = app
        .oneshot(common::post_request("/api/v1/items", request_body))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let item: serde_json::Value = common::response_json(response).await;
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
        .oneshot(common::post_request("/api/v1/items", request_body))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let item: serde_json::Value = common::response_json(response).await;
    assert_eq!(item["name"], "Test Item");
    assert!(item["description"].is_null());
}

#[tokio::test]
#[ignore = "Validation not yet implemented"]
async fn test_create_item_missing_name() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "description": "Missing name"
    });

    let response = app
        .oneshot(common::post_request("/api/v1/items", request_body))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: serde_json::Value = common::response_json(response).await;
    assert_eq!(error["error"], "VALIDATION_ERROR");
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

// READ tests
#[tokio::test]
async fn test_get_item() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create an item first
    let created =
        common::create_test_item(&state.repo, "Test Item", Some("Test Description")).await;

    let response = app
        .oneshot(common::get_request(&format!("/api/v1/items/{}", created.id)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let item: serde_json::Value = common::response_json(response).await;
    assert_eq!(item["id"], created.id);
    assert_eq!(item["name"], "Test Item");
    assert_eq!(item["description"], "Test Description");
}

#[tokio::test]
async fn test_get_nonexistent_item() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/api/v1/items/nonexistent"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// UPDATE tests
#[tokio::test]
async fn test_update_item() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create an item first
    let created =
        common::create_test_item(&state.repo, "Original Name", Some("Original Description")).await;

    let update_body = json!({
        "name": "Updated Name"
    });

    let response = app
        .oneshot(common::put_request(&format!("/api/v1/items/{}", created.id), update_body))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let item: serde_json::Value = common::response_json(response).await;
    assert_eq!(item["name"], "Updated Name");
    assert_eq!(item["description"], "Original Description");
}

#[tokio::test]
async fn test_update_nonexistent_item() {
    let app = common::create_test_app().await;

    let update_body = json!({
        "name": "New Name"
    });

    let response = app
        .oneshot(common::put_request("/api/v1/items/nonexistent", update_body))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// DELETE tests
#[tokio::test]
async fn test_delete_item() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create an item first
    let created = common::create_test_item(&state.repo, "To Delete", None).await;

    let response = app
        .clone()
        .oneshot(common::delete_request(&format!("/api/v1/items/{}", created.id)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify item is deleted
    let response = app
        .oneshot(common::get_request(&format!("/api/v1/items/{}", created.id)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_item() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::delete_request("/api/v1/items/nonexistent"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// LIST tests
#[tokio::test]
async fn test_list_items() {
    let state = common::create_test_state();
    let app = estuary::routes::create_routes(state.clone());

    // Create multiple items
    common::create_test_items(&state.repo, 5).await;

    let response = app
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let list_response: serde_json::Value = common::response_json(response).await;
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
    common::create_test_items(&state.repo, 5).await;

    let response = app
        .oneshot(common::get_request("/api/v1/items?limit=2&offset=2"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let list_response: serde_json::Value = common::response_json(response).await;
    assert_eq!(list_response["items"].as_array().unwrap().len(), 2);
    assert_eq!(list_response["total"], 5);
    assert_eq!(list_response["limit"], 2);
    assert_eq!(list_response["offset"], 2);
}

#[tokio::test]
#[ignore = "Query validation not yet implemented"]
async fn test_invalid_pagination_params() {
    let app = common::create_test_app().await;

    // Test negative limit
    let response = app
        .clone()
        .oneshot(common::get_request("/api/v1/items?limit=-1"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test limit exceeding max
    let response = app
        .oneshot(common::get_request("/api/v1/items?limit=1000"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Rate limiting tests
#[tokio::test]
async fn test_rate_limit_headers() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("X-RateLimit-Limit"));
    assert!(response.headers().contains_key("X-RateLimit-Remaining"));
    assert!(response.headers().contains_key("X-RateLimit-Reset"));
}

#[tokio::test]
async fn test_rate_limit_multiple_requests() {
    let app = common::create_test_app().await;

    // First request
    let response = app
        .clone()
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let initial_remaining = response
        .headers()
        .get("X-RateLimit-Remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u32>()
        .unwrap();

    // Second request
    let response = app
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let new_remaining = response
        .headers()
        .get("X-RateLimit-Remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u32>()
        .unwrap();

    assert_eq!(new_remaining, initial_remaining - 1);
}

// Security headers tests
#[tokio::test]
async fn test_security_headers() {
    let app = common::create_test_app().await;

    let response = app.oneshot(common::get_request("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check security headers
    assert_eq!(response.headers().get("X-Content-Type-Options").unwrap(), "nosniff");
    assert_eq!(response.headers().get("X-Frame-Options").unwrap(), "DENY");
    assert!(response.headers().contains_key("X-Request-Id"));
}

// Error response tests
#[tokio::test]
async fn test_structured_error_response_format() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/api/v1/items/nonexistent"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: serde_json::Value = common::response_json(response).await;

    // Check error response structure
    assert!(error["error"].is_string());
    assert!(error["message"].is_string());
    assert!(error["timestamp"].is_string());
    // request_id is optional in error responses
    if error.get("request_id").is_some() {
        assert!(error["request_id"].is_string());
    }
}

#[tokio::test]
async fn test_validation_error_structure() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from("{invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: serde_json::Value = common::response_json(response).await;
    assert_eq!(error["error"], "BAD_REQUEST");
    assert!(error["message"].as_str().unwrap().contains("JSON"));
}

#[tokio::test]
async fn test_error_codes_match_status_codes() {
    let app = common::create_test_app().await;

    // 404 Not Found
    let response = app
        .clone()
        .oneshot(common::get_request("/api/v1/items/nonexistent"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let error: serde_json::Value = common::response_json(response).await;
    assert_eq!(error["error"], "NOT_FOUND");

    // 400 Bad Request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from("not json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error: serde_json::Value = common::response_json(response).await;
    assert_eq!(error["error"], "BAD_REQUEST");
}

// OpenAPI tests
#[tokio::test]
async fn test_openapi_json_endpoint() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/openapi.json"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_json::<serde_json::Value>(response).await;

    // Check OpenAPI structure
    assert_eq!(body["openapi"], "3.1.0");
    assert_eq!(body["info"]["title"], "Estuary API");
    assert_eq!(body["info"]["version"], env!("CARGO_PKG_VERSION"));

    // Check that paths are defined
    assert!(body["paths"].is_object());
    assert!(body["paths"]["/health"].is_object());
    assert!(body["paths"]["/api/v1/items"].is_object());
    assert!(body["paths"]["/api/v1/items/{id}"].is_object());

    // Check components
    assert!(body["components"]["schemas"].is_object());
    assert!(body["components"]["schemas"]["Item"].is_object());
    assert!(body["components"]["schemas"]["CreateItemRequest"].is_object());
    assert!(body["components"]["schemas"]["ErrorResponse"].is_object());
}
