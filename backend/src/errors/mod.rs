use axum::{
    Json,
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::{error, warn};

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

// Convert DbErr → ApiError
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

// Convert StatusCode → ApiError
impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        ApiError::new(status, status.canonical_reason().unwrap_or("Unknown error"))
    }
}

// Convert MultipartError → ApiError
impl From<MultipartError> for ApiError {
    fn from(err: MultipartError) -> Self {
        warn!("{}", err);
        ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("Invalid multipart request: {}", err),
        )
    }
}

// Convert serde_json Error → ApiError
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("Invalid JSON payload: {}", err),
        )
    }
}

impl From<aws_sdk_s3::Error> for ApiError {
    fn from(err: aws_sdk_s3::Error) -> Self {
        eprintln!("{:#?}", err);

        ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("Invalid S3 request: {}", err),
        )
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        eprintln!("{:#?}", err);

        ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("std::io::Error request: {}", err),
        )
    }
}
