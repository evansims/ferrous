use crate::database::DatabaseError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InternalServerError(String),
    BadRequest(String),
    LockError,
    DatabaseError(DatabaseError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::LockError => write!(f, "Failed to acquire lock"),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg)
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::LockError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "lock_error",
                "Failed to acquire lock".to_string(),
            ),
            AppError::DatabaseError(e) => match e {
                DatabaseError::NotFound => (
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "Resource not found".to_string(),
                ),
                DatabaseError::ConnectionError(msg) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "database_connection_error",
                    msg,
                ),
                DatabaseError::QueryError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_query_error",
                    msg,
                ),
                DatabaseError::SerializationError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "serialization_error",
                    msg,
                ),
                DatabaseError::LockError => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "lock_error",
                    "Failed to acquire database lock".to_string(),
                ),
            },
        };

        let error_response = ErrorResponse {
            error: error_code.to_string(),
            message,
        };

        (status, Json(error_response)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        AppError::DatabaseError(error)
    }
}
