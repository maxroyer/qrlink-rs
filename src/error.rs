use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Application-wide error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Link not found")]
    LinkNotFound,

    #[error("Link has expired")]
    LinkExpired,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded(u64),

    #[error("Failed to generate short code after multiple attempts")]
    ShortCodeExhausted,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("QR generation failed: {0}")]
    QrGeneration(String),

    #[error("Internal server error")]
    Internal(String),
}

/// Error response body sent to clients.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, message) = match &self {
            AppError::LinkNotFound => (StatusCode::NOT_FOUND, "not_found", None),
            AppError::LinkExpired => (StatusCode::GONE, "link_expired", None),
            AppError::InvalidUrl(msg) => (StatusCode::BAD_REQUEST, "invalid_url", Some(msg.clone())),
            AppError::RateLimitExceeded(retry_after) => {
                let body = ErrorResponse {
                    error: "rate_limit_exceeded".to_string(),
                    message: Some(format!("Retry after {} seconds", retry_after)),
                };
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(header::RETRY_AFTER, retry_after.to_string())],
                    Json(body),
                )
                    .into_response();
            }
            AppError::ShortCodeExhausted => {
                (StatusCode::SERVICE_UNAVAILABLE, "short_code_exhausted", None)
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", None)
            }
            AppError::QrGeneration(msg) => {
                tracing::error!("QR generation error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "qr_error", Some(msg.clone()))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", None)
            }
        };

        let body = ErrorResponse {
            error: error.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}

/// Result type alias for application operations.
pub type AppResult<T> = Result<T, AppError>;
