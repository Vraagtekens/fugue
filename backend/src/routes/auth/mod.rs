use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub mod handlers;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/test", get(handlers::test))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login::login))
}
