use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{info_span, Instrument};
use uuid::Uuid;

/// Header name for request ID
pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Request ID middleware
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

/// Request ID extractor
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

/// Tower layer for request ID middleware
#[derive(Clone)]
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

/// Tower service for request ID middleware
#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for RequestIdService<S>
where
    S: Service<Request<B>> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        // Generate or extract request ID
        let request_id = if let Some(existing_id) = req.headers().get(&X_REQUEST_ID) {
            existing_id.to_str().unwrap_or_default().to_string()
        } else {
            Uuid::new_v4().to_string()
        };

        // Add to extensions
        req.extensions_mut().insert(RequestId(request_id));

        self.inner.call(req)
    }
}
