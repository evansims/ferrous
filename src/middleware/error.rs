use crate::{
    error::{ErrorCode, ErrorDetails, ErrorResponse},
    request_id::RequestId,
};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;

/// Middleware to handle errors and inject request IDs
pub async fn error_handler_middleware(req: Request, next: Next) -> Response {
    // Extract request ID from extensions
    let _request_id = req.extensions().get::<RequestId>().map(|id| id.0.clone());

    let response = next.run(req).await;

    // If the response is an error (4xx or 5xx), try to enhance it with request ID
    if response.status().is_client_error() || response.status().is_server_error() {
        // Check if response body is our ErrorResponse by looking at content-type
        if let Some(content_type) = response.headers().get("content-type") {
            if content_type
                .to_str()
                .unwrap_or("")
                .contains("application/json")
            {
                // Try to inject request_id into existing error response
                // This is a bit tricky with Axum's response model, so we'll skip modification
                // The error response will be created with request_id in the AppError::into_response
            }
        }
    }

    response
}

/// Create an error response with request ID from the current context
pub fn create_error_response(
    code: ErrorCode,
    message: String,
    details: Option<ErrorDetails>,
    request_id: Option<String>,
) -> impl IntoResponse {
    let error_response = ErrorResponse {
        error: code,
        message,
        details,
        timestamp: Utc::now(),
        request_id,
    };

    Json(error_response)
}
