use crate::{errors::ApiError, middleware::auth_middleware::require_auth, state::AppState};
use axum::{Router, http::StatusCode, middleware};

pub mod auth;
pub mod sessions;

pub fn create_routes(state: AppState) -> Router<AppState> {
    let auth_routes = auth::auth_routes();

    // let protected_sessions = sessions::sessions_routes();
    let protected_sessions = sessions::sessions_routes()
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/sessions", protected_sessions)
        .fallback(handler_404)
}

async fn handler_404() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "Route not found")
}
