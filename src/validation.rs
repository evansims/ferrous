use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationError, ValidationErrors};

/// A custom extractor that validates JSON payloads
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ValidationRejection::Json)?;

        value.validate().map_err(ValidationRejection::Validation)?;

        Ok(ValidatedJson(value))
    }
}

/// Custom rejection type for validation errors
#[derive(Debug)]
pub enum ValidationRejection {
    Json(JsonRejection),
    Validation(ValidationErrors),
}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        use crate::error::{
            ErrorCode, ErrorDetails, ErrorResponse, ValidationError as ErrorValidation,
        };
        use chrono::Utc;

        let (status, error_response) = match self {
            ValidationRejection::Json(rejection) => {
                let message = match rejection {
                    JsonRejection::JsonDataError(_) => "Invalid JSON format",
                    JsonRejection::JsonSyntaxError(_) => "Malformed JSON",
                    JsonRejection::MissingJsonContentType(_) => {
                        "Missing Content-Type: application/json header"
                    }
                    _ => "Bad request",
                };

                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        error: ErrorCode::BadRequest,
                        message: message.to_string(),
                        details: None,
                        timestamp: Utc::now(),
                        request_id: None, // Will be injected by middleware
                    },
                )
            }
            ValidationRejection::Validation(errors) => {
                let validation_errors: Vec<ErrorValidation> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |e| ErrorValidation {
                            field: field.to_string(),
                            message: e
                                .message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| e.code.to_string()),
                            code: Some(e.code.to_string()),
                        })
                    })
                    .collect();

                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorResponse {
                        error: ErrorCode::ValidationError,
                        message: "Validation failed".to_string(),
                        details: Some(ErrorDetails {
                            validation_errors: Some(validation_errors),
                            context: None,
                        }),
                        timestamp: Utc::now(),
                        request_id: None, // Will be injected by middleware
                    },
                )
            }
        };

        (status, Json(error_response)).into_response()
    }
}

/// Validator for checking string length
pub fn validate_length_range(
    min: usize,
    max: usize,
) -> impl Fn(&str) -> Result<(), ValidationError> {
    move |s: &str| {
        let len = s.trim().len();
        if len < min || len > max {
            let mut error = ValidationError::new("length");
            error.message = Some(std::borrow::Cow::Owned(if min == 0 {
                format!("Must be at most {} characters", max)
            } else if len < min {
                format!("Must be at least {} characters", min)
            } else {
                format!("Must be between {} and {} characters", min, max)
            }));
            return Err(error);
        }
        Ok(())
    }
}

/// Validator for non-empty trimmed strings
pub fn validate_not_empty(s: &str) -> Result<(), ValidationError> {
    if s.trim().is_empty() {
        let mut error = ValidationError::new("empty");
        error.message = Some(std::borrow::Cow::Borrowed("Cannot be empty"));
        return Err(error);
    }
    Ok(())
}

/// Sanitize string by trimming whitespace
pub fn sanitize_string(s: String) -> String {
    s.trim().to_string()
}

/// Sanitize optional string
pub fn sanitize_optional_string(s: Option<String>) -> Option<String> {
    s.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
