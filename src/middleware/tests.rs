#[cfg(test)]
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    Router,
};
use tower::ServiceExt;

async fn test_middleware(req: Request<Body>, next: Next) -> Response {
    let mut res = next.run(req).await;
    res.headers_mut()
        .insert("x-test", "middleware-applied".parse().unwrap());
    res
}

async fn blocking_middleware(_req: Request<Body>, _next: Next) -> Result<Response, StatusCode> {
    Err(StatusCode::FORBIDDEN)
}

#[tokio::test]
async fn test_middleware_chain_execution() {
    let app = Router::new()
        .route("/", axum::routing::get(|| async { "Hello" }))
        .layer(middleware::from_fn(test_middleware));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("x-test").unwrap(), "middleware-applied");
}

#[tokio::test]
async fn test_middleware_ordering() {
    // Middleware is applied in reverse order (last added = first executed)
    let app = Router::new()
        .route("/", axum::routing::get(|| async { "Hello" }))
        .layer(middleware::from_fn(|req: Request<Body>, next: Next| async move {
            let mut res = next.run(req).await;
            res.headers_mut().insert("x-second", "2".parse().unwrap());
            res
        }))
        .layer(middleware::from_fn(|req: Request<Body>, next: Next| async move {
            let mut res = next.run(req).await;
            res.headers_mut().insert("x-first", "1".parse().unwrap());
            res
        }));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // The "first" middleware is actually executed second
    assert_eq!(response.headers().get("x-first").unwrap(), "1");
    assert_eq!(response.headers().get("x-second").unwrap(), "2");
}

#[tokio::test]
async fn test_blocking_middleware() {
    let app = Router::new()
        .route("/", axum::routing::get(|| async { "Hello" }))
        .layer(middleware::from_fn(blocking_middleware));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_response_modification_in_middleware() {
    let app = Router::new()
        .route("/", axum::routing::get(|| async { (StatusCode::OK, "Original") }))
        .layer(middleware::from_fn(|req: Request<Body>, next: Next| async move {
            let mut res = next.run(req).await;
            // Modify response body
            *res.status_mut() = StatusCode::ACCEPTED;
            res
        }));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}
