use crate::common;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limit_headers() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check rate limit headers
    assert!(response.headers().contains_key("x-ratelimit-limit"));
    assert!(response.headers().contains_key("x-ratelimit-remaining"));
    assert!(response.headers().contains_key("x-ratelimit-reset"));
}

#[tokio::test]
async fn test_rate_limit_multiple_requests() {
    let app = common::create_test_app().await;

    // Make first request
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let limit = response
        .headers()
        .get("x-ratelimit-limit")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let remaining = response
        .headers()
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u32>()
        .unwrap();

    // Remaining should be limit - 1
    assert_eq!(remaining, limit - 1);

    // Make second request
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let remaining2 = response
        .headers()
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u32>()
        .unwrap();

    // Remaining should decrease
    assert_eq!(remaining2, limit - 2);
}

// Note: Testing actual rate limit exceeded would require making many requests
// or mocking the rate limiter, which is beyond the scope of this basic test
