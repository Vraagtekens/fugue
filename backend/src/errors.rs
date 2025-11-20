use axum::{
    BoxError, Json,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        let msg = message.into();
        // Log the error as soon as it is created
        error!("API Error: {} - {}", status, msg);
        Self {
            status,
            message: msg,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Optional: log again when converting to response
        error!("Responding with error: {} - {}", self.status, self.message);
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(e: sea_orm::DbErr) -> Self {
        match e {
            sea_orm::DbErr::RecordNotFound(_) => {
                ApiError::new(StatusCode::NOT_FOUND, "Record not found")
            }
            sea_orm::DbErr::Conn(_) => {
                ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "Database unavailable")
            }
            other => ApiError::new(StatusCode::BAD_REQUEST, other.to_string()),
        }
    }
}

use axum::Error;

// pub async fn global_error_handler(err: Error) -> ApiError {
//     // Axum extractor errors land here
//     ApiError::new(StatusCode::BAD_REQUEST, err.to_string())
// }

pub async fn global_error_handler(
    // `Method` and `Uri` are extractors so they can be used here
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("`{method} {uri}` failed with {err}"),
    )
}

pub async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {err}"),
        )
    }
}
