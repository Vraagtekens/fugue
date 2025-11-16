use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub mod handlers;

pub fn sessions_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_pomodoro_sessions))
        .route("/add", post(handlers::add))
}
