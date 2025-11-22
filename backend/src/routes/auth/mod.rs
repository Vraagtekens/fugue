use crate::errors::ApiError;
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Router, ServiceExt};

pub mod handlers;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/test", get(handlers::test))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login::login))
}
