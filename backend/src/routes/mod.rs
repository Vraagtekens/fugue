use crate::{middleware::auth_middleware::require_auth, state::AppState};
use axum::{Router, middleware};

pub mod auth;
pub mod sessions;

pub fn create_routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = auth::auth_routes();

    let protected_sessions = sessions::sessions_routes().layer(middleware::from_fn_with_state(
        app_state.clone(),
        require_auth,
    ));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/sessions", protected_sessions)
}
