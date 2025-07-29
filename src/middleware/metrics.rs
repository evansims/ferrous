use crate::metrics::{track_http_request, Timer};
use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Middleware to track HTTP request metrics
pub async fn metrics_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let timer = Timer::new();
    let method = req.method().to_string();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(req).await;
    let status = response.status().as_u16();
    let duration = timer.elapsed_seconds();

    // Track the request
    track_http_request(&method, &path, status, duration);

    Ok(response)
}
