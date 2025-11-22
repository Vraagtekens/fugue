use std::convert::Infallible;

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

use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};

// ---------------------
// NEWTYPE REJECTION WRAPPERS
// ---------------------

pub struct JsonError(pub JsonRejection);
pub struct QueryError(pub QueryRejection);
pub struct PathError(pub PathRejection);

// Convert JsonRejection → ApiError → Response
impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        ApiError::new(StatusCode::BAD_REQUEST, self.0.body_text()).into_response()
    }
}

impl IntoResponse for QueryError {
    fn into_response(self) -> Response {
        ApiError::new(StatusCode::BAD_REQUEST, self.0.body_text()).into_response()
    }
}

impl IntoResponse for PathError {
    fn into_response(self) -> Response {
        ApiError::new(StatusCode::BAD_REQUEST, self.0.body_text()).into_response()
    }
}
