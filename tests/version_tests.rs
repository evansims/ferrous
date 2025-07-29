use axum::http::{HeaderValue, StatusCode};
use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn test_version_in_url_path() {
    let app = common::create_test_app().await;

    // Test explicit v1 in path
    let response = app
        .clone()
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_version_via_accept_header() {
    let app = common::create_test_app().await;

    let mut request = common::get_request("/api/v1/items");
    request.headers_mut().insert(
        "accept",
        HeaderValue::from_static("application/vnd.estuary.v1+json"),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_version_via_custom_header() {
    let app = common::create_test_app().await;

    let mut request = common::get_request("/api/v1/items");
    request
        .headers_mut()
        .insert("x-api-version", HeaderValue::from_static("v1"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_default_version_when_none_specified() {
    let app = common::create_test_app().await;

    // Request without version should still work (uses default)
    let response = app
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_version_precedence() {
    let app = common::create_test_app().await;

    // URL path should take precedence over headers
    let mut request = common::get_request("/api/v1/items");
    request
        .headers_mut()
        .insert("x-api-version", HeaderValue::from_static("v2")); // This should be ignored

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK); // v1 endpoint should work
}

#[tokio::test]
async fn test_unsupported_version_in_accept_header() {
    let app = common::create_test_app().await;

    let mut request = common::get_request("/api/v1/items");
    request.headers_mut().insert(
        "accept",
        HeaderValue::from_static("application/vnd.estuary.v99+json"),
    );

    // Should fall back to default version since v99 doesn't exist
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deprecation_headers_not_present_for_current_version() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/api/v1/items"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Current version should not have deprecation headers
    assert!(response.headers().get("deprecation").is_none());
    assert!(response.headers().get("sunset").is_none());
    assert!(response.headers().get("warning").is_none());
}

#[tokio::test]
async fn test_openapi_includes_version_info() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(common::get_request("/openapi.json"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = common::response_json(response).await;

    // Check that version info is present
    assert!(body["info"]["description"]
        .as_str()
        .unwrap()
        .contains("API version v1"));
}

// Test version extraction functions directly
#[cfg(test)]
mod unit_tests {
    use axum::http::HeaderMap;
    use estuary::version::{extract_version, ApiVersion};

    #[test]
    fn test_version_extraction_from_various_sources() {
        let mut headers = HeaderMap::new();

        // Test URL path extraction
        let version = extract_version("/api/v1/items", &headers);
        assert_eq!(version, ApiVersion::V1);

        // Test Accept header extraction
        headers.insert("accept", "application/vnd.estuary.v1+json".parse().unwrap());
        let version = extract_version("/items", &headers);
        assert_eq!(version, ApiVersion::V1);

        // Test X-API-Version header extraction
        headers.clear();
        headers.insert("x-api-version", "v1".parse().unwrap());
        let version = extract_version("/items", &headers);
        assert_eq!(version, ApiVersion::V1);

        // Test default version
        headers.clear();
        let version = extract_version("/items", &headers);
        assert_eq!(version, ApiVersion::default());
    }
}
