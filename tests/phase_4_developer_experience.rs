use axum::http::StatusCode;
use serde_json::Value;
use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn test_openapi_json_endpoint() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/openapi.json"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = common::response_json::<Value>(response).await;

    // Check OpenAPI structure
    assert_eq!(body["openapi"], "3.1.0");
    assert_eq!(body["info"]["title"], "Estuary API");
    assert_eq!(body["info"]["version"], env!("CARGO_PKG_VERSION"));

    // Check that paths are defined
    assert!(body["paths"].is_object());
    assert!(body["paths"]["/health"].is_object());
    assert!(body["paths"]["/api/v1/items"].is_object());
    assert!(body["paths"]["/api/v1/items/{id}"].is_object());

    // Check that components are defined
    assert!(body["components"]["schemas"].is_object());
    assert!(body["components"]["schemas"]["Item"].is_object());
    assert!(body["components"]["schemas"]["ErrorResponse"].is_object());
}

#[tokio::test]
async fn test_structured_error_response_format() {
    let app = common::create_test_app().await;

    // Test 404 error
    let response = app
        .oneshot(common::get_request("/api/v1/items/non-existent-id"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = common::response_json::<Value>(response).await;

    // Check error response structure
    assert!(body["error"].is_string());
    assert_eq!(body["error"], "NOT_FOUND");
    assert!(body["message"].is_string());
    assert!(body["timestamp"].is_string());
    // request_id field might not be present if null (due to skip_serializing_if)
}

#[tokio::test]
async fn test_validation_error_structure() {
    let app = common::create_test_app().await;

    // Create item with empty name (should fail validation)
    let create_request = common::post_request(
        "/api/v1/items",
        serde_json::json!({
            "name": "",
            "description": "Test description"
        }),
    );

    let response = app.oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = common::response_json::<Value>(response).await;

    // Check validation error structure
    assert_eq!(body["error"], "VALIDATION_ERROR");
    assert_eq!(body["message"], "Validation failed");
    assert!(body["details"].is_object());
    assert!(body["details"]["validation_errors"].is_array());

    let validation_errors = &body["details"]["validation_errors"];
    assert!(!validation_errors.as_array().unwrap().is_empty());

    // Check individual validation error structure
    let first_error = &validation_errors[0];
    assert!(first_error["field"].is_string());
    assert!(first_error["message"].is_string());
}

#[tokio::test]
async fn test_error_codes_match_status_codes() {
    let app = common::create_test_app().await;

    // Test various error scenarios
    let scenarios = vec![
        (
            "/api/v1/items/non-existent",
            "GET",
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
        ),
        (
            "/api/v1/items?limit=200",
            "GET",
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
        ),
    ];

    for (path, method, expected_status, expected_code) in scenarios {
        let request = match method {
            "GET" => common::get_request(path),
            _ => panic!("Unsupported method"),
        };

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), expected_status);

        let body = common::response_json::<Value>(response).await;
        assert_eq!(body["error"], expected_code);
    }
}
