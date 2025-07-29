use crate::common;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_security_headers() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check security headers
    assert_eq!(
        response.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");
    assert_eq!(
        response.headers().get("x-xss-protection").unwrap(),
        "1; mode=block"
    );
    assert_eq!(
        response.headers().get("referrer-policy").unwrap(),
        "strict-origin-when-cross-origin"
    );
    assert!(response.headers().contains_key("content-security-policy"));
    assert!(response.headers().contains_key("permissions-policy"));

    // Should not have HSTS in development mode
    assert!(!response.headers().contains_key("strict-transport-security"));
}
