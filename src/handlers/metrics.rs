use crate::metrics::get_metrics;
use axum::{
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
};

/// Prometheus metrics endpoint
pub async fn metrics_handler() -> Result<Response, StatusCode> {
    let metrics = get_metrics();

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, "text/plain; version=0.0.4")],
        metrics,
    )
        .into_response())
}
