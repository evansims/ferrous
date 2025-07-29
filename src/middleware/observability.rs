use crate::metrics::{track_http_request, Timer};
use axum::{
    body::Body,
    extract::{MatchedPath, Request},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info_span, Instrument};
use uuid::Uuid;

/// Header name for request ID
pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Request ID extractor for use in handlers
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl<S> axum::extract::FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| RequestId(Uuid::new_v4().to_string()));

        Ok(request_id)
    }
}

/// Request ID middleware - generates or propagates request IDs
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    // Check if request already has a request ID
    let request_id = if let Some(existing_id) = req.headers().get(&X_REQUEST_ID) {
        existing_id.to_str().unwrap_or_default().to_string()
    } else {
        // Generate new request ID
        Uuid::new_v4().to_string()
    };

    // Add request ID to request extensions
    req.extensions_mut().insert(RequestId(request_id.clone()));

    // Create span with request ID for structured logging
    let span = info_span!(
        "request",
        request_id = %request_id,
        method = %req.method(),
        uri = %req.uri(),
    );

    // Process request within the span
    let mut response = next.run(req).instrument(span).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(X_REQUEST_ID.clone(), header_value);
    }

    response
}

/// Metrics middleware - tracks HTTP request metrics
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
