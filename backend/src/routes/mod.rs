use crate::{errors::ApiError, routes::docs::ApiDoc, state::AppState};
use axum::{Router, http::StatusCode};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod docs;
pub mod sessions;

pub fn create_routes(state: AppState) -> Router<AppState> {
    let auth_routes = auth::auth_routes();

    // let protected_sessions = sessions::sessions_routes();
    // let protected_sessions = sessions::sessions_routes().route_layer(
    // middleware::from_fn_with_state(state.clone(), require_jwt),
    let protected_sessions = sessions::sessions_routes(state.clone());

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/sessions", protected_sessions)
        .nest(
            "/docs",
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi())
                .into(),
        )
        .fallback(handler_404)
}

async fn handler_404() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "Route not found")
}
