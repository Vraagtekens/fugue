use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, patch, post};

use crate::state::AppState;

pub mod handlers;

pub fn sessions_routes(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/", get(handlers::get_sessions))
        .route(
            "/add",
            post(handlers::add).layer(DefaultBodyLimit::max(50 * 1024 * 1024)), // 50 MB
        )
        .route("/live/record", get(handlers::live::record_ws))
        .route(
            "/live/subscribe/{session_id}",
            get(handlers::live::subscribe_ws),
        )
        .route("/{id}", delete(handlers::delete_session))
        .route("/{id}/favorite", patch(handlers::set_favorite))
        // .route("/{*wildcard}", get(handlers::get_session_midi))
        .route("/mp3/{*wildcard}", get(handlers::get_session_midi_mp3))
        .route("/audio/{*wildcard}", get(handlers::get_session_audio))
        .route("/pdf/{*wildcard}", get(handlers::get_session_midi_pdf))
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::auth_middleware::require_api_key,
        ));

    protected.merge(Router::new().route("/live/subscribe", get(handlers::live::subscribe_all_ws)))
}
