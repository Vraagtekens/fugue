use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

use crate::state::AppState;

pub mod handlers;

pub fn sessions_routes() -> Router<AppState> {
    Router::new().route("/", get(handlers::get_sessions)).route(
        "/add",
        post(handlers::add).layer(DefaultBodyLimit::max(50 * 1024 * 1024)), // 50 MB
    )

    // .route("/", get(handlers::test))
}
