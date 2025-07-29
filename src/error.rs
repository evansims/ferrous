use crate::database::DatabaseError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json; // Used in #[schema(example = json!({...}))] attributes
use std::fmt;
use utoipa::ToSchema;

/// Standard error response structure
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "error": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": {
        "validation_errors": [{
            "field": "name",
            "message": "Name must be between 1 and 255 characters",
            "code": "length"
        }]
    },
    "timestamp": "2024-01-01T00:00:00Z",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
}))]
pub struct ErrorResponse {
    /// Machine-readable error code
    pub error: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
    /// Timestamp of the error
    pub timestamp: DateTime<Utc>,
    /// Request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Detailed error information
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetails {
    /// Field-specific validation errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<ValidationError>>,
    /// Additional context about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Individual validation error
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "field": "name",
    "message": "Name must be between 1 and 255 characters",
    "code": "length"
}))]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Machine-readable error codes
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Client errors (4xx)
    BadRequest,
    ValidationError,
    NotFound,
    Unauthorized,
    Forbidden,
    RateLimitExceeded,

    // Server errors (5xx)
    InternalServerError,
    DatabaseError,
    LockError,
    ServiceUnavailable,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InternalServerError(String),
    BadRequest(String),
    ValidationError(String),
    LockError,
    DatabaseError(DatabaseError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::LockError => write!(f, "Failed to acquire lock"),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Try to extract request ID from the current request context
        let request_id = None; // Will be populated by middleware

        let (status, error_code, message, details) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorCode::NotFound, msg, None),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
                msg,
                None,
            ),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, ErrorCode::BadRequest, msg, None)
            }
            AppError::ValidationError(msg) => {
                // Try to parse validation errors for field-specific details
                let details = parse_validation_errors(&msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorCode::ValidationError,
                    "Validation failed".to_string(),
                    Some(ErrorDetails {
                        validation_errors: details,
                        context: Some(msg),
                    }),
                )
            }
            AppError::LockError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::LockError,
                "Failed to acquire lock".to_string(),
                None,
            ),
            AppError::DatabaseError(e) => match e {
                DatabaseError::NotFound => (
                    StatusCode::NOT_FOUND,
                    ErrorCode::NotFound,
                    "Resource not found".to_string(),
                    None,
                ),
                DatabaseError::ConnectionError(msg) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ErrorCode::ServiceUnavailable,
                    "Database connection error".to_string(),
                    Some(ErrorDetails {
                        validation_errors: None,
                        context: Some(msg),
                    }),
                ),
                DatabaseError::QueryError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::DatabaseError,
                    "Database query error".to_string(),
                    Some(ErrorDetails {
                        validation_errors: None,
                        context: Some(msg),
                    }),
                ),
                DatabaseError::SerializationError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalServerError,
                    "Data serialization error".to_string(),
                    Some(ErrorDetails {
                        validation_errors: None,
                        context: Some(msg),
                    }),
                ),
                DatabaseError::LockError => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::LockError,
                    "Failed to acquire database lock".to_string(),
                    None,
                ),
            },
        };

        let error_response = ErrorResponse {
            error: error_code,
            message,
            details,
            timestamp: Utc::now(),
            request_id,
        };

        (status, Json(error_response)).into_response()
    }
}

/// Parse validation error string to extract field-specific errors
fn parse_validation_errors(error_str: &str) -> Option<Vec<ValidationError>> {
    // Simple parsing for validator crate output
    // Format: "field_name: error_message"
    let errors: Vec<ValidationError> = error_str
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some(ValidationError {
                    field: parts[0].trim().to_string(),
                    message: parts[1].trim().to_string(),
                    code: None,
                })
            } else {
                None
            }
        })
        .collect();

    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        AppError::DatabaseError(error)
    }
}

impl From<crate::validation::ValidationRejection> for AppError {
    fn from(rejection: crate::validation::ValidationRejection) -> Self {
        match rejection {
            crate::validation::ValidationRejection::Json(_) => {
                AppError::BadRequest("Invalid JSON format".to_string())
            }
            crate::validation::ValidationRejection::Validation(errors) => {
                AppError::ValidationError(errors.to_string())
            }
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        // For query parameter validation, we want to return BadRequest
        AppError::BadRequest(errors.to_string())
    }
}
